use crate::AppState;
use crate::SaveMsg;
use chrono::DateTime;
use chrono::Datelike;
use chrono::FixedOffset;
use chrono::TimeDelta;
use chrono::Timelike;
use chrono::Utc;
use fitsio::images::ImageDescription;
use fitsio::images::ImageType;
use fitsio::FitsFile;
use isvp_sys::*;
use log::{error, info, warn};
use minimp4::Mp4Muxer;
use mxu::create_mask;
use opencv::core::*;
use opencv::imgcodecs::imencode_def;
use opencv::imgproc::cvt_color_def;
use opencv::imgproc::line;
use opencv::imgproc::LineTypes;
use opencv::imgproc::COLOR_YUV2RGB_NV21;
use solver::*;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

pub fn record_loops(
    detected_rx: mpsc::Receiver<SaveMsg>,
    app_state: Arc<Mutex<AppState>>,
    flag: Arc<AtomicBool>,
) {
    info!("Starting hevc_loop");
    let res = thread::Builder::new()
        .name("hevc_loop".to_string())
        .spawn(move || unsafe { get_h264_stream(app_state, flag) });

    if let Err(e) = res {
        error!("Failed to start hevc_loop : {}", e);
        panic!();
    }

    info!("Starting save_detection_loop");
    let res = thread::Builder::new()
        .name("save_detection_loop".to_string())
        .spawn(move || save_detection(detected_rx));

    if let Err(e) = res {
        error!("Failed to start save_detection_loop : {}", e);
        panic!();
    }
}

unsafe fn get_h264_stream(app_state: Arc<Mutex<AppState>>, flag: Arc<AtomicBool>) {
    if IMP_Encoder_StartRecvPic(3) < 0 {
        error!("IMP_Encoder_StartRecvPic failed");
        panic!();
    }

    loop {
        if flag.load(Ordering::Relaxed) {
            log::info!("Stopping hevc_loop");
            break;
        }

        let app_state_tmp = match app_state.lock() {
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
        let now: DateTime<Utc> = Utc::now();
        let offset = FixedOffset::east_opt(app_state_tmp.timezone).unwrap();
        let current_time: DateTime<FixedOffset> = now.with_timezone(&offset);
        drop(app_state_tmp);
        let next_minute = current_time.minute() + 1;

        let (adjusted_hour, adjusted_minute) = if next_minute >= 60 {
            (current_time.hour() + 1, 0)
        } else {
            (current_time.hour(), next_minute)
        };

        let (adjusted_day, adjusted_hour_final) = if adjusted_hour >= 24 {
            (current_time.day() + 1, 0)
        } else {
            (current_time.day(), adjusted_hour)
        };

        let target_time = current_time
            .with_day(adjusted_day)
            .unwrap()
            .with_hour(adjusted_hour_final)
            .unwrap()
            .with_minute(adjusted_minute)
            .unwrap()
            .with_second(0)
            .unwrap();

        let dir_day = current_time
            .format("/media/mmc/records/regular/%Y%m%d")
            .to_string();

        let dir = Path::new(&dir_day);
        if !dir.is_dir() {
            if let Err(e) = std::fs::create_dir(dir) {
                error!("create_dir failed : {}", e);
                panic!();
            }
        }

        let hour_dir = dir.join(format!("{:02}", current_time.hour()));
        if !hour_dir.is_dir() {
            if let Err(e) = std::fs::create_dir(&hour_dir) {
                error!("create_dir failed : {}", e);
                panic!();
            }
        }

        let mp4path = current_time
            .format("/media/mmc/records/regular/%Y%m%d/%H/%M.mp4")
            .to_string();

        let mp4title = current_time.format("%Y-%m-%d-%H-%M").to_string();

        let file = match File::create(&mp4path) {
            Ok(f) => f,
            Err(e) => {
                error!("create failed : {}", e);
                panic!()
            }
        };
        let mut mp4muxer = Mp4Muxer::new(file);
        mp4muxer.init_video(1920, 1080, true, &mp4title);
        if IMP_Encoder_RequestIDR(3) < 0 {
            log::error!("IMP_Encoder_RequestIDR failed");
            break;
        }

        loop {
            let app_state_tmp = match app_state.lock() {
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
            let now: DateTime<Utc> = Utc::now();
            let offset = FixedOffset::east_opt(app_state_tmp.timezone).unwrap();
            let current_time: DateTime<FixedOffset> = now.with_timezone(&offset);
            let fps = app_state_tmp.fps;
            drop(app_state_tmp);

            if current_time >= target_time {
                break;
            } else if target_time - current_time >= TimeDelta::minutes(1) {
                // if system time was set to past.
                break;
            }

            if flag.load(Ordering::Relaxed) {
                log::info!("Stopping hevc_loop");
                break;
            }

            if IMP_Encoder_PollingStream(3, 10000) < 0 {
                warn!("IMP_Encoder_PollingStream failed");
                continue;
            }

            let mut stream = IMPEncoderStream {
                phyAddr: 0,
                virAddr: 0,
                streamSize: 0,
                pack: std::ptr::null_mut(),
                packCount: 0,
                isVI: false,
                seq: 0,
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

            if IMP_Encoder_GetStream(3, &mut stream, false) < 0 {
                error!("IMP_Encoder_GetStream failed");
                panic!();
            }

            let stream_packs = std::slice::from_raw_parts(
                stream.pack as *const IMPEncoderPack,
                stream.packCount as usize,
            );

            for pack in stream_packs {
                if pack.length > 0 {
                    let rem_size = stream.streamSize - pack.offset;
                    if rem_size < pack.length {
                        let src1 = std::slice::from_raw_parts(
                            (stream.virAddr + pack.offset) as *const u8,
                            rem_size as usize,
                        );
                        mp4muxer.write_video_with_fps(src1, fps);
                        let src2 = std::slice::from_raw_parts(
                            stream.virAddr as *const u8,
                            (pack.length - rem_size) as usize,
                        );
                        mp4muxer.write_video_with_fps(src2, fps);
                    } else {
                        let src = std::slice::from_raw_parts(
                            (stream.virAddr + pack.offset) as *const u8,
                            pack.length as usize,
                        );
                        mp4muxer.write_video_with_fps(src, fps);
                    }
                }
            }

            if IMP_Encoder_ReleaseStream(3, &mut stream) < 0 {
                error!("IMP_Encoder_ReleaseStream failed");
                panic!();
            }
        }

        mp4muxer.close();
    }
}

fn u8_to_u32(src: &[u8], dst: &mut Vec<u32>) {
    for i in 0..src.len() {
        dst[i] = i as u32;
    }
}

unsafe fn try_solve_field(field: &mut Vec<u8>, mask_small: &[u8]) -> Option<Solved> {
    let mut mask_full = Vec::with_capacity(field.len());
    create_mask(mask_small.as_ptr(), mask_full.as_mut_ptr());

    let mut field_u32 = Vec::with_capacity(field.len());
    field_u32.set_len(field.len());
    u8_to_u32(&field, &mut field_u32);
    let catalog = match extract(&mut field_u32, Some(&mask_full), 640, 360) {
        Some(c) => c,
        None => {
            warn!("Nothing found in catalog");
            return None;
        }
    };
    match solve_field(catalog, true) {
        Some(solved) => Some(solved),
        None => {
            warn!("Nothing found in field");
            None
        }
    }
}

fn save_detection(rx: mpsc::Receiver<SaveMsg>) {
    while let Ok(msg) = rx.recv() {
        match msg {
            SaveMsg::Detection(mut msg) => {
                let dir_path = msg
                    .time
                    .format("/media/mmc/records/detected/%Y-%m-%d_%H_%M_%S")
                    .to_string();

                let dir = Path::new(&dir_path);
                if let Err(e) = std::fs::create_dir(dir) {
                    error!("create_dir failed : {}", e);
                    panic!();
                }

                let comp = unsafe {
                    Mat::new_rows_cols_with_data_def(
                        1080 + 540,
                        1920,
                        CV_8UC1,
                        msg.data.as_mut_ptr() as *mut ::std::os::raw::c_void,
                    )
                    .unwrap()
                };

                let mut colored = Mat::default();
                // Actually, it has to be NV12 but somehow this works.
                cvt_color_def(&comp, &mut colored, COLOR_YUV2RGB_NV21).unwrap();
                let mut jpeg_buf = Vector::<u8>::new();
                imencode_def(".jpg", &colored, &mut jpeg_buf).unwrap();

                let path = msg
                    .time
                    .format("/media/mmc/records/detected/%Y-%m-%d_%H_%M_%S/detected.jpg")
                    .to_string();
                let file = match File::create(&path) {
                    Ok(f) => f,
                    Err(e) => {
                        error!("create failed : {}", e);
                        panic!();
                    }
                };
                let mut writer = BufWriter::new(file);
                if let Err(e) = writer.write_all(jpeg_buf.as_slice()) {
                    error!("jpeg write_all failed : {}", e);
                    panic!();
                }
                if let Err(e) = writer.into_inner().unwrap().sync_all() {
                    error!("jpeg into_inner failed : {}", e);
                    panic!();
                }

                let path = msg
                    .time
                    .format("/media/mmc/records/detected/%Y-%m-%d_%H_%M_%S/detected.fits")
                    .to_string();
                let image_description = ImageDescription {
                    data_type: ImageType::UnsignedByte,
                    dimensions: &[1080 + 540, 1920],
                };

                let mut fptr = match FitsFile::create(&path)
                    .with_custom_primary(&image_description)
                    .overwrite()
                    .open()
                {
                    Ok(f) => f,
                    Err(e) => {
                        error!("FitsFile create failed : {}", e);
                        panic!();
                    }
                };

                let hdu = fptr.primary_hdu().unwrap();

                if msg.solve_field && (msg.save_wcs || msg.draw_constellation) {
                    let mut field = msg.field.take().unwrap();
                    let mask_small = msg.mask.take().unwrap();
                    info!("Trying solve field");
                    let solved_ = unsafe { try_solve_field(&mut field, &mask_small) };
                    if let Some(wcs) = solved_ {
                        if msg.save_wcs {
                            unsafe {
                                if let Err(e) = wcs.save_to_hdu(&hdu, &mut fptr) {
                                    error!("Failed to save wcs : {}", e);
                                    panic!();
                                }
                            }
                        }

                        if msg.draw_constellation {
                            let star_lines = unsafe { wcs.get_constellations() };
                            for (p1, p2) in star_lines {
                                let pt1 = Point {
                                    x: (p1.0 * 3.).round() as i32,
                                    y: (p1.1.round() * 3.) as i32,
                                };
                                let pt2 = Point {
                                    x: (p2.0 * 3.).round() as i32,
                                    y: (p2.1.round() * 3.) as i32,
                                };
                                line(
                                    &mut colored,
                                    pt1,
                                    pt2,
                                    Scalar::new(255.0, 255.0, 255.0, 0.0),
                                    2,
                                    LineTypes::LINE_AA as i32,
                                    0,
                                )
                                .unwrap();
                            }

                            let mut jpeg_buf = Vector::<u8>::new();
                            imencode_def(".jpg", &colored, &mut jpeg_buf).unwrap();

                            let path = msg
                                .time
                                .format(
                                    "/media/mmc/records/detected/%Y-%m-%d_%H_%M_%S/anotated.jpg",
                                )
                                .to_string();
                            let file = match File::create(&path) {
                                Ok(f) => f,
                                Err(e) => {
                                    error!("create failed : {}", e);
                                    panic!();
                                }
                            };
                            let mut writer = BufWriter::new(file);
                            if let Err(e) = writer.write_all(jpeg_buf.as_slice()) {
                                error!("constellation write_all failed : {}", e);
                                panic!();
                            }
                            if let Err(e) = writer.into_inner().unwrap().sync_all() {
                                error!("constellation into_inner failed : {}", e);
                                panic!();
                            }
                        }
                    } else {
                        warn!("Failed solve field");
                    }
                }

                if let Err(e) = hdu.write_image(&mut fptr, &msg.data) {
                    error!("Failed to save detected fits : {}", e);
                    panic!();
                }
            }
            SaveMsg::Capture(mut msg) => {
                let comp = unsafe {
                    Mat::new_rows_cols_with_data_def(
                        1080 + 540,
                        1920,
                        CV_8UC1,
                        msg.data.as_mut_ptr() as *mut ::std::os::raw::c_void,
                    )
                    .unwrap()
                };

                let mut colored = Mat::default();
                // Actually, it has to be NV12 but somehow this works.
                cvt_color_def(&comp, &mut colored, COLOR_YUV2RGB_NV21).unwrap();
                let mut jpeg_buf = Vector::<u8>::new();
                imencode_def(".jpg", &colored, &mut jpeg_buf).unwrap();

                let path = msg
                    .time
                    .format("/media/mmc/records/capture/%Y-%m-%d_%H_%M_%S.jpg")
                    .to_string();
                let file = match File::create(&path) {
                    Ok(f) => f,
                    Err(e) => {
                        error!("create failed : {}", e);
                        panic!();
                    }
                };
                let mut writer = BufWriter::new(file);
                if let Err(e) = writer.write_all(jpeg_buf.as_slice()) {
                    error!("capture write_all failed : {}", e);
                    panic!();
                }
                if let Err(e) = writer.into_inner().unwrap().sync_all() {
                    error!("capture into_inner failed : {}", e);
                    panic!();
                }

                let path = msg
                    .time
                    .format("/media/mmc/records/capture/%Y-%m-%d_%H_%M_%S.fits")
                    .to_string();
                let image_description = ImageDescription {
                    data_type: ImageType::UnsignedByte,
                    dimensions: &[1080 + 540, 1920],
                };

                let mut fptr = match FitsFile::create(&path)
                    .with_custom_primary(&image_description)
                    .overwrite()
                    .open()
                {
                    Ok(f) => f,
                    Err(e) => {
                        error!("FitsFile create failed : {}", e);
                        panic!();
                    }
                };

                let hdu = fptr.primary_hdu().unwrap();

                if let Err(e) = hdu.write_image(&mut fptr, &msg.data) {
                    error!("Failed to save detected fits : {}", e);
                    panic!();
                }
            }
        }
    }
}
