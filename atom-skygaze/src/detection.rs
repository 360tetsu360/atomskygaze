use chrono::*;
//use fitsio::images::ImageDescription;
//use fitsio::images::ImageType;
//use fitsio::FitsFile;
use isvp_sys::*;
use log::error;
use opencv::core::*;
//use opencv::imgcodecs::imencode;
//use opencv::imgcodecs::imencode_def;
//use opencv::imgproc;
//use opencv::imgproc::*;
use opencv::prelude::FastLineDetectorTrait;
use opencv::ximgproc::create_fast_line_detector;
use std::collections::VecDeque;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Cursor;
use std::io::Write;
use std::os::raw::c_void;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::{mpsc, watch};

type DetectionSender = Arc<Mutex<Vec<(DateTime<FixedOffset>, DateTime<FixedOffset>)>>>;

const RESIZED_X: usize = 32;
const RESIZED_Y: usize = 18;

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

    fn vertices(&self) -> [(i32, i32); 4] {
        [
            (self.x, self.y),
            (self.x + self.width, self.y),
            (self.x, self.y + self.height),
            (self.x + self.width, self.y + self.height),
        ]
    }

    fn min_distance(&self, other: &Blob) -> i32 {
        let mut min_distance = i32::MAX;
        let mut min_vertices = ((0, 0), (0, 0));

        if self.is_overlapping(other) {
            return 0;
        }

        for &vertex1 in self.vertices().iter() {
            for &vertex2 in other.vertices().iter() {
                let distance = (vertex1.0 - vertex2.0).abs() + (vertex1.1 - vertex2.1).abs();
                if distance < min_distance {
                    min_distance = distance;
                    min_vertices = (vertex1, vertex2);
                }
            }
        }

        (min_vertices.0 .0 - min_vertices.1 .0).pow(2)
            + (min_vertices.0 .1 - min_vertices.1 .1).pow(2)
    }

    fn bounding_box(&self) -> (i32, i32, i32, i32) {
        (self.x, self.y, self.x + self.width, self.y + self.height)
    }

    fn is_overlapping(&self, other: &Blob) -> bool {
        let (x1, y1, x2, y2) = self.bounding_box();
        let (x3, y3, x4, y4) = other.bounding_box();

        // 二つの四角形が重なっているかどうかを確認する
        !(x2 <= x3 || x4 <= x1 || y2 <= y3 || y4 <= y1)
    }

    fn update(&mut self, other: &Blob) {
        let min_x = self.x.min(other.x);
        let min_y = self.y.min(other.y);
        let max_x = (self.x + self.width).max(other.x + other.width);
        let max_y = (self.y + self.height).max(other.y + other.height);

        self.x = min_x;
        self.y = min_y;
        self.width = max_x - min_x;
        self.height = max_y - min_y;
    }
}

pub unsafe fn init() -> bool {
    if IMP_FrameSource_SetFrameDepth(0, 2) < 0 {
        error!("IMP_FrameSource_SetFrameDepth failed");
        return false;
    }

    if IMP_FrameSource_SetFrameDepth(1, 2) < 0 {
        error!("IMP_FrameSource_SetFrameDepth failed");
        return false;
    }

    true
}

pub unsafe fn jpeg_start(tx: watch::Sender<Vec<u8>>) -> bool {
    if IMP_Encoder_StartRecvPic(2) < 0 {
        error!("IMP_Encoder_StartRecvPic failed");
        return false;
    }

    loop {
        if IMP_Encoder_PollingStream(2, 10000) < 0 {
            error!("IMP_Encoder_PollingStream failed");
            return false;
        }

        let mut stream = IMPEncoderStream {
            phyAddr: 0,
            virAddr: 0,
            streamSize: 0,
            pack: std::ptr::null_mut(),
            packCount: 0,
            seq: 0,
            isVI: false,
            __bindgen_anon_1: IMPEncoderStream__bindgen_ty_1 {
                streamInfo: IMPEncoderStreamInfo {
                    iNumBytes: 0,
                    uNumIntra: 0,
                    uNumSkip: 0,
                    uNumCU8x8: 0,
                    uNumCU16x16: 0,
                    uNumCU32x32: 0,
                    uNumCU64x64: 0,
                    iSliceQP: 0,
                    iMinQP: 0,
                    iMaxQP: 0,
                },
            },
        };

        if IMP_Encoder_GetStream(2, &mut stream, true) < 0 {
            error!("IMP_Encoder_GetStream failed");
            return false;
        }

        let stream_packs = std::slice::from_raw_parts(
            stream.pack as *const IMPEncoderPack,
            stream.packCount as usize,
        );
        for pack in stream_packs {
            if pack.length > 0 {
                let jpeg_buffer = vec![];
                let mut cursor = Cursor::new(jpeg_buffer);
                let rem_size = stream.streamSize - pack.offset;
                if rem_size < pack.length {
                    let src1 = std::slice::from_raw_parts(
                        (stream.virAddr + pack.offset) as *const u8,
                        rem_size as usize,
                    );
                    cursor.write_all(src1).unwrap();
                    let src2 = std::slice::from_raw_parts(
                        stream.virAddr as *const u8,
                        (pack.length - rem_size) as usize,
                    );
                    cursor.write_all(src2).unwrap();
                } else {
                    let src = std::slice::from_raw_parts(
                        (stream.virAddr + pack.offset) as *const u8,
                        pack.length as usize,
                    );
                    cursor.write_all(src).unwrap();
                }

                tx.send(cursor.into_inner()).unwrap();
            }
        }

        if IMP_Encoder_ReleaseStream(2, &mut stream) < 0 {
            error!("IMP_Encoder_ReleaseStream failed");
            return false;
        }
    }
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

fn get_move_area(diff: &Mat, mask: &[u8]) -> Vec<Blob> {
    let mut resized = vec![];
    for y in 0..18 {
        let mut new_row = vec![];
        for x in 0..32 {
            let roi = Rect::new(x * 20, y * 20, 20, 20);
            let cropped = Mat::roi(diff, roi).unwrap();
            let (mean, stddev): (f64, f64) = {
                let mut mean = Scalar_::default();
                let mut stddev = Scalar_::default();
                mean_std_dev(&cropped, &mut mean, &mut stddev, &no_array()).unwrap();
                (mean[0], stddev[0])
            };

            //println!("stddev {}, mean {}", stddev, mean);
            let thresh_val = mean + stddev * 2.;

            let bin = if thresh_val > 30. && mask[(y * 32 + x) as usize] != 1 {
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

fn composite(diff_list: &mut VecDeque<Mat>) -> Mat {
    let mut res = diff_list.pop_front().unwrap();
    while let Some(diff_list) = diff_list.pop_front() {
        let mut result = Mat::default();
        opencv::core::add(&res, &diff_list, &mut result, &no_array(), -1).unwrap();
        res = result;
    }

    res
}

pub unsafe fn save_stream() {
    let mut cnt = 0;
    let mut last_time = std::time::Instant::now();
    loop {
        let fd = File::create(&format!("/media/mmc/tmp/{}", cnt)).unwrap();
        let mut writer = std::io::BufWriter::new(fd);
        let mut full_frame: *mut IMPFrameInfo = std::ptr::null_mut();

        IMP_FrameSource_GetFrame(0, &mut full_frame);
        let img_buffer = std::slice::from_raw_parts(
            (*full_frame).virAddr as *const u8,
            (*full_frame).size as usize,
        );
        writer.write_all(img_buffer).unwrap();
        println!(
            "{}",
            std::time::Instant::now()
                .duration_since(last_time)
                .as_micros()
        );
        println!(
            "s {}, w {}, h {}",
            (*full_frame).size,
            (*full_frame).width,
            (*full_frame).height
        );
        last_time = std::time::Instant::now();
        IMP_FrameSource_ReleaseFrame(0, full_frame);
        writer.flush().unwrap();
        cnt += 1;
    }
}

pub unsafe fn start(
    mut mrx: mpsc::Receiver<Vec<u8>>,
    detecting: Arc<Mutex<bool>>, /*, tx: watch::Sender<Vec<u8>>*/
    sender: DetectionSender,
) {
    let mut last_frame_time = 0;
    let mut last_frame: *mut IMPFrameInfo = std::ptr::null_mut();
    let mut last_gray = Mat::default();
    let mut diff_list = VecDeque::<Mat>::with_capacity(10);
    let mut mask = vec![0u8; 32 * 18];
    let mut detection_start: DateTime<FixedOffset> =
        Utc::now().with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap());

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("/media/mmc/meteor_log.txt")
        .unwrap();

    let mut detecting_flag = 0;

    let _index = 0u128;
    loop {
        let mut new_frame: *mut IMPFrameInfo = std::ptr::null_mut();
        if let Ok(msk) = mrx.try_recv() {
            mask = msk;
        }

        IMP_FrameSource_GetFrame(1, &mut new_frame);
        last_frame_time = (*new_frame).timeStamp;

        let (img_width, img_height) = ((*new_frame).width, (*new_frame).height);
        let new_gray = Mat::new_rows_cols_with_data_def(
            img_height as i32,
            img_width as i32,
            CV_8UC1,
            (*new_frame).virAddr as *mut c_void,
        )
        .unwrap();

        if *detecting.lock().unwrap() && !last_frame.is_null() {
            let mut diff = Mat::default();

            absdiff(&new_gray, &last_gray, &mut diff).unwrap();
            //let mut jpeg_buff = Vector::<u8>::new();
            //imencode_def(".jpg", &diff, &mut jpeg_buff).unwrap();
            //tx.send(jpeg_buff.to_vec()).unwrap();

            diff_list.push_back(diff);

            if diff_list.len() == 10 {
                let integrated_diff = composite(&mut diff_list);
                let boxes = get_move_area(&integrated_diff, &mask);
                //println!("{}", boxes.len());
                for rect in boxes.iter() {
                    if rect.pix_cnt > 20 {
                        println!("Too Big Roi!");
                        continue;
                    }

                    let cropped = integrated_diff.roi(rect.to_rect_with_scalar(20)).unwrap();
                    let mut lines: Vector<Vec4i> = Vector::new();
                    let mut fld = create_fast_line_detector(10, 1.732, 33., 66., 3, true).unwrap();
                    fld.detect(&cropped, &mut lines).unwrap();

                    if !lines.is_empty() {
                        let now: DateTime<Utc> = Utc::now();
                        let time: DateTime<FixedOffset> =
                            now.with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap());
                        if detecting_flag == 0 {
                            detection_start = time - Duration::seconds(2);
                        }
                        println!("[{}] Meteor Detected", time);
                        writeln!(file, "[{}] detected", time).unwrap();
                        detecting_flag = 25;
                    }
                }
            }

            if detecting_flag != 0 {
                detecting_flag -= 1;
                println!("{}", detecting_flag);

                if detecting_flag == 0 {
                    println!("saving detection");
                    let now: DateTime<Utc> = Utc::now();
                    let time: DateTime<FixedOffset> =
                        now.with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap());
                    sender.lock().unwrap().push((detection_start, time));
                }
            }
        }

        IMP_FrameSource_ReleaseFrame(1, last_frame);
        last_frame = new_frame;
        last_gray = new_gray;
    }
}

//fn save_nv12_tofits(nv12_buffer: &[u8], _path: &Path) {}
