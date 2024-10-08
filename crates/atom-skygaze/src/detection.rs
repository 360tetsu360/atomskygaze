use crate::AppState;
use crate::LogType;
use chrono::*;
use isvp_sys::*;
use log::{error, info, warn};
use mxu::*;
use opencv::core::*;
use opencv::prelude::FastLineDetectorTrait;
use opencv::ximgproc::create_fast_line_detector;
use std::collections::VecDeque;
use std::os::raw::c_void;
use std::slice::from_raw_parts;
use std::sync::{Arc, Mutex, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::watch;

const RESIZED_X: usize = 32;
const RESIZED_Y: usize = 18;
const ROI_SIZE: i32 = 20;

#[derive(Copy, Clone, Debug)]
struct Blob {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    pix_cnt: usize,
}

impl Blob {
    fn new(x: i32, y: i32, width: i32, height: i32, pix_cnt: usize) -> Self {
        Self {
            x,
            y,
            width,
            height,
            pix_cnt,
        }
    }

    fn to_rect_with_scalar(self, scalar: i32) -> Rect {
        Rect::new(
            self.x * scalar,
            self.y * scalar,
            self.width * scalar,
            self.height * scalar,
        )
    }
}

pub unsafe fn init() -> bool {
    if IMP_FrameSource_SetFrameDepth(0, 3) < 0 {
        error!("IMP_FrameSource_SetFrameDepth failed");
        return false;
    }

    if IMP_FrameSource_SetFrameDepth(1, 2) < 0 {
        error!("IMP_FrameSource_SetFrameDepth failed");
        return false;
    }

    true
}

struct Point {
    x: usize,
    y: usize,
}

fn find_bounding_rectangles(image: &Vec<Vec<u8>>) -> Vec<Blob> {
    let mut bounding_rectangles = Vec::new();
    let mut visited = vec![vec![false; RESIZED_X]; RESIZED_Y];

    // Depth-first search to find connected components
    for y in 0..RESIZED_Y {
        for x in 0..RESIZED_X {
            if image[y][x] == 1 && !visited[y][x] {
                let mut min = Point {
                    x: usize::MAX,
                    y: usize::MAX,
                };

                let mut max = Point {
                    x: usize::MIN,
                    y: usize::MIN,
                };

                let mut pix_cnt = 0;
                dfs(
                    image,
                    &mut visited,
                    Point { x, y },
                    &mut min,
                    &mut max,
                    &mut pix_cnt,
                );
                bounding_rectangles.push(Blob::new(
                    min.x as i32,
                    min.y as i32,
                    (max.x - min.x + 1) as i32,
                    (max.y - min.y + 1) as i32,
                    pix_cnt,
                ));
            }
        }
    }

    bounding_rectangles
}

fn dfs(
    image: &Vec<Vec<u8>>,
    visited: &mut Vec<Vec<bool>>,
    pos: Point,
    min: &mut Point,
    max: &mut Point,
    pix_cnt: &mut usize,
) {
    let x = pos.x;
    let y = pos.y;
    if x >= image[0].len() || y >= image.len() || image[y][x] == 0 || visited[y][x] {
        return;
    }

    visited[y][x] = true;
    min.x = min.x.min(x);
    min.y = min.y.min(y);
    max.x = max.x.max(x);
    max.y = max.y.max(y);
    *pix_cnt += 1;

    let deltas = [
        (-1, 0),
        (1, 0),
        (0, -1),
        (0, 1),
        (-1, -1),
        (-1, 1),
        (1, -1),
        (1, 1),
    ]; // 8 directions
    for &(dx, dy) in &deltas {
        let new_x = (x as i32 + dx) as usize;
        let new_y = (y as i32 + dy) as usize;
        dfs(
            image,
            visited,
            Point { x: new_x, y: new_y },
            min,
            max,
            pix_cnt,
        );
    }
}

fn get_move_area(
    diff: &Mat,
    mask: &[u8],
    overall_mean: f64,
    overall_stddev: f64,
    threshold: f64,
) -> Vec<Blob> {
    let mut resized = vec![];
    for y in 0..18 {
        let mut new_row = vec![];
        for x in 0..32 {
            let roi = Rect::new(x * ROI_SIZE, y * ROI_SIZE, ROI_SIZE, ROI_SIZE);
            let cropped = Mat::roi(diff, roi).unwrap();
            let mean: f64 = mean_def(&cropped).unwrap()[0];

            let z_score = (mean - overall_mean) / overall_stddev;

            let bin = if z_score > threshold && mask[(y * 32 + x) as usize] != 1 {
                1u8
            } else {
                0u8
            };
            new_row.push(bin);
        }
        resized.push(new_row);
    }

    find_bounding_rectangles(&resized)
}

unsafe fn integrate(diff_list: &mut VecDeque<Vec<u8>>) -> Vec<u8> {
    let mut res = diff_list.pop_front().unwrap();
    while let Some(diff) = diff_list.pop_front() {
        buffer_add(res.as_ptr(), diff.as_ptr(), res.as_mut_ptr(), res.len());
    }

    res
}

unsafe fn composite(comp_list: &mut VecDeque<Vec<u8>>) -> Vec<u8> {
    // NV12 type
    let mut res = Vec::with_capacity(1920 * (1080 + 540));
    let frame_bufs: Vec<*const u8> = comp_list.iter().map(|buf| buf.as_ptr()).collect();
    buffer_max_list(
        frame_bufs.as_ptr(),
        res.as_mut_ptr(),
        comp_list.len(),
        1920 * (1080 + 540),
    );
    res.set_len(1920 * (1080 + 540));

    res
}

fn is_within_time_range(now: DateTime<FixedOffset>, start: (u32, u32), end: (u32, u32)) -> bool {
    let current_minutes = now.hour() * 60 + now.minute();

    let start_minutes = start.0 * 60 + start.1;
    let end_minutes = end.0 * 60 + end.1;

    if start_minutes <= end_minutes {
        current_minutes >= start_minutes && current_minutes <= end_minutes
    } else {
        current_minutes >= start_minutes || current_minutes <= end_minutes
    }
}

pub enum SaveMsg {
    Detection(DetectionMsg),
    Capture(CaptureMsg),
}

pub struct DetectionMsg {
    pub data: Vec<u8>,
    pub time: DateTime<FixedOffset>,
    pub solve_field: bool,
    pub field: Option<Vec<u8>>,
    pub mask: Option<Vec<u8>>,
    pub save_wcs: bool,
    pub draw_constellation: bool,
}

pub struct CaptureMsg {
    pub data: Vec<u8>,
    pub time: DateTime<FixedOffset>,
}

pub unsafe fn start(
    app_state: Arc<Mutex<AppState>>,
    sender: mpsc::Sender<SaveMsg>,
    log_tx: watch::Sender<LogType>,
    flag: Arc<AtomicBool>,
) {
    let mut last_frame: *mut IMPFrameInfo = std::ptr::null_mut();
    let mut diff_list = VecDeque::<Vec<u8>>::with_capacity(10);
    let mut comp_list = VecDeque::<Vec<u8>>::with_capacity(10);
    let mut stack_frame: Option<Vec<u8>> = None;
    let app_state_tmp = app_state.lock().unwrap();
    let offset = FixedOffset::east_opt(app_state_tmp.timezone).unwrap();
    let mut detection_start: DateTime<FixedOffset> = Utc::now().with_timezone(&offset);

    drop(app_state_tmp);

    let mut detecting_flag = 0;

    let _index = 0u128;
    loop {
        if flag.load(Ordering::Relaxed) {
            log::info!("Stopping detection_loop");
            break;
        }

        let mut new_frame: *mut IMPFrameInfo = std::ptr::null_mut();
        let mut full_frame: *mut IMPFrameInfo = std::ptr::null_mut();

        IMP_FrameSource_GetFrame(0, &mut full_frame);
        IMP_FrameSource_GetFrame(1, &mut new_frame);

        let (img_width, img_height) = ((*new_frame).width, (*new_frame).height);

        let mut app_state_tmp = match app_state.lock() {
            Ok(guard) => guard,
            Err(e) => {
                warn!(
                    "app_state mutex lock error : {} at {}:{}",
                    e,
                    file!(),
                    line!()
                );
                continue;
            }
        };
        if app_state_tmp.cap {
            app_state_tmp.cap = false;
            let offset = FixedOffset::east_opt(app_state_tmp.timezone).unwrap();

            let time: DateTime<FixedOffset> = Utc::now().with_timezone(&offset);
            let mut frame = Vec::with_capacity(1920 * (1080 + 540));
            fast_memcpy(
                (*full_frame).virAddr as *const u8,
                frame.as_mut_ptr(),
                1920 * (1080 + 540),
            );

            sender
                .send(SaveMsg::Capture(CaptureMsg { data: frame, time }))
                .unwrap();
        }

        let is_detect_time = if app_state_tmp.detection_config.use_time {
            let det_time = app_state_tmp.detection_config.detection_time;
            let offset = FixedOffset::east_opt(app_state_tmp.timezone).unwrap();
            let now = Utc::now().with_timezone(&offset);
            is_within_time_range(now, det_time.start, det_time.end)
        } else {
            true
        };

        if app_state_tmp.detect && !last_frame.is_null() && is_detect_time {
            let mut diff = Vec::with_capacity(640 * 360);

            // MXU2.0 SIMD128
            // four time faster than OpenCV absdiff().
            buffer_absdiff(
                (*new_frame).virAddr as *const u8,
                (*last_frame).virAddr as *const u8,
                diff.as_mut_ptr(),
                (img_height * img_width) as usize,
            );

            diff.set_len(640 * 360);

            diff_list.push_back(diff);

            // NV12 type
            let mut frame = Vec::with_capacity(1920 * (1080 + 540));
            fast_memcpy(
                (*full_frame).virAddr as *const u8,
                frame.as_mut_ptr(),
                1920 * (1080 + 540),
            );
            frame.set_len(1920 * (1080 + 540));
            comp_list.push_back(frame);

            if diff_list.len() > (app_state_tmp.fps / 5) as usize {
                let mut diff_buff = integrate(&mut diff_list);

                let mut mean: f64 = 0.;
                let mut stddev: f64 = 0.;
                fast_mean_stddev(
                    diff_buff.as_ptr(),
                    (img_height * img_width) as usize,
                    &mut mean,
                    &mut stddev,
                );

                let integrated_diff = Mat::new_rows_cols_with_data_def(
                    img_height as i32,
                    img_width as i32,
                    CV_8UC1,
                    diff_buff.as_mut_ptr() as *mut c_void,
                )
                .unwrap();

                let boxes = get_move_area(
                    &integrated_diff,
                    &app_state_tmp.mask,
                    mean,
                    stddev,
                    app_state_tmp.detection_config.threshold,
                );
                //println!("{}", boxes.len());
                for rect in boxes.iter() {
                    if rect.pix_cnt > app_state_tmp.detection_config.max_roi_size {
                        warn!("he size of the ROI exceeds the detectable upper limit");
                        continue;
                    }

                    let cropped = integrated_diff
                        .roi(rect.to_rect_with_scalar(ROI_SIZE))
                        .unwrap();
                    let mut lines: Vector<Vec4i> = Vector::new();
                    let mut fld = create_fast_line_detector(
                        app_state_tmp.detection_config.length_threshold as i32,
                        app_state_tmp.detection_config.distance_threshold,
                        33.,
                        66.,
                        3,
                        true,
                    )
                    .unwrap();
                    fld.detect(&cropped, &mut lines).unwrap();

                    if !lines.is_empty() {
                        let offset = FixedOffset::east_opt(app_state_tmp.timezone).unwrap();

                        let time: DateTime<FixedOffset> = Utc::now().with_timezone(&offset);
                        if detecting_flag == 0 {
                            stack_frame = Some(composite(&mut comp_list));
                            detection_start = time;
                        }
                        info!("Meteor Detected");
                        detecting_flag = app_state_tmp.fps;
                    }
                }
            }

            while comp_list.len() > (app_state_tmp.fps / 5) as usize {
                comp_list.pop_front();
            }

            if detecting_flag != 0 {
                if let Some(stack_frame_mut) = stack_frame.as_mut() {
                    //let st = std::time::Instant::now();
                    lighten_stack(
                        (*full_frame).virAddr as *const u8,
                        stack_frame_mut.as_mut_ptr(),
                        stack_frame_mut.len(),
                    );
                }

                detecting_flag -= 1;
                if detecting_flag == 0 {
                    //sender.send(None).unwrap();
                    info!("Start saving detection");

                    let (field, mask) = if app_state_tmp.detection_config.solve_field
                        && (app_state_tmp.detection_config.save_wcs
                            || app_state_tmp.detection_config.draw_constellation)
                    {
                        let field = from_raw_parts(
                            (*last_frame).virAddr as *const u8,
                            (img_width * img_height) as usize,
                        )
                        .to_vec();
                        let mask = app_state_tmp.mask.clone();
                        (Some(field), Some(mask))
                    } else {
                        (None, None)
                    };

                    sender
                        .send(SaveMsg::Detection(DetectionMsg {
                            data: stack_frame.take().unwrap(),
                            time: detection_start,
                            solve_field: app_state_tmp.detection_config.solve_field,
                            field,
                            mask,
                            save_wcs: app_state_tmp.detection_config.save_wcs,
                            draw_constellation: app_state_tmp.detection_config.draw_constellation,
                        }))
                        .unwrap();

                    let fractional_second =
                        (detection_start.timestamp_subsec_millis() as f64) / 100.0;
                    let timestamp = format!(
                        "{}.{}",
                        detection_start.format("%Y-%m-%d %H:%M:%S"),
                        fractional_second as i32
                    );

                    let jpgpath = detection_start
                        .format("%Y-%m-%d_%H_%M_%S/detected.jpg")
                        .to_string();

                    let log = LogType::Detection(timestamp, jpgpath);

                    let log_clone = log.clone();
                    log_tx.send(log_clone).unwrap();
                    app_state_tmp.logs.push(log)
                }
            }
        }
        drop(app_state_tmp);

        IMP_FrameSource_ReleaseFrame(0, full_frame);
        IMP_FrameSource_ReleaseFrame(1, last_frame);
        last_frame = new_frame;
    }
}

//fn save_nv12_tofits(nv12_buffer: &[u8], _path: &Path) {}
