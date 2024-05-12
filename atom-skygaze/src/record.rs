use crate::AppState;
use chrono::DateTime;
use chrono::FixedOffset;
use chrono::{Datelike, Duration, Local, Timelike};
use isvp_sys::*;
use log::error;
use minimp4::Mp4Muxer;
use std::fs::File;
use std::path::Path;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use chrono::TimeDelta;
use std::io::BufWriter;
use opencv::core::*;
use opencv::imgcodecs::imencode_def;
use opencv::imgproc::cvt_color_def;
use opencv::imgproc::COLOR_YUV2RGB_NV21;
use std::io::Write;
use fitsio::images::ImageDescription;
use fitsio::images::ImageType;
use fitsio::FitsFile;

pub fn mp4save_loops(
    detected_rx: mpsc::Receiver<(Vec<u8>, DateTime<FixedOffset>)>,
    app_state: Arc<Mutex<AppState>>,
    flag: Arc<Mutex<bool>>,
) {
    let (tx, rx) = mpsc::channel();

    thread::Builder::new()
        .name("h264_loop".to_string())
        .spawn(move || unsafe { get_h264_stream(tx, app_state, flag) })
        .unwrap();

    thread::Builder::new()
        .name("filesave_loop".to_string())
        .spawn(move || sync_file(rx))
        .unwrap();

    thread::Builder::new()
        .name("save_detection_loop".to_string())
        .spawn(move || save_detection(detected_rx))
        .unwrap();
}

unsafe fn get_h264_stream(
    tx: mpsc::Sender<File>,
    app_state: Arc<Mutex<AppState>>,
    flag: Arc<Mutex<bool>>,
) -> bool {

    if IMP_Encoder_StartRecvPic(3) < 0 {
        error!("IMP_Encoder_StartRecvPic failed");
        return false;
    }

    loop {
        let shutdown_flag = match flag.lock() {
            Ok(guard) => guard,
            Err(_) => continue,
        };

        if *shutdown_flag {
            break true;
        }
        drop(shutdown_flag);

        let current_time = Local::now() + Duration::hours(9);
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
            std::fs::create_dir(dir).unwrap();
        }

        let hour_dir = dir.join(format!("{:02}", current_time.hour()));
        if !hour_dir.is_dir() {
            std::fs::create_dir(&hour_dir).unwrap();
        }

        let mp4path = current_time
            .format("/media/mmc/records/regular/%Y%m%d/%H/%M.mp4")
            .to_string();

        let mp4title = current_time.format("%Y-%m-%d-%H-%M").to_string();

        let file = File::create(mp4path).unwrap();
        let mut mp4muxer = Mp4Muxer::new(file);
        mp4muxer.init_video(1920, 1080, true, &mp4title);

        loop {
            let current_time = Local::now() + Duration::hours(9);
            if current_time >= target_time {
                break;
            } else if target_time - current_time >= TimeDelta::minutes(1) {
                // if system time was set to past.
                break;
            }

            let shutdown_flag = match flag.lock() {
                Ok(guard) => guard,
                Err(_) => continue,
            };

            if *shutdown_flag {
                break;
            }
            drop(shutdown_flag);

            let app_state_tmp = match app_state.lock() {
                Ok(guard) => guard,
                Err(_) => continue,
            };
            let fps = app_state_tmp.fps;
            drop(app_state_tmp);

            if IMP_Encoder_PollingStream(3, 10000) < 0 {
                error!("IMP_Encoder_PollingStream failed");
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
                return false;
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
                return false;
            }

        }

        let file = mp4muxer.close();
        tx.send(file).unwrap();
    }
}

fn sync_file(rx: mpsc::Receiver<File>) {
    while let Ok(file) = rx.recv() {
        file.sync_all().unwrap();
        println!("Saved mp4");
    }
}

fn save_detection(rx: mpsc::Receiver<(Vec<u8>, DateTime<FixedOffset>)>) {
    while let Ok((mut frame, time)) = rx.recv() {
        let comp = unsafe {Mat::new_rows_cols_with_data_def(
            (1080 + 540) as i32,
            1920 as i32,
            CV_8UC1,
            frame.as_mut_ptr() as *mut ::std::os::raw::c_void,
        ).unwrap()};

        let mut colored = Mat::default();
        cvt_color_def(&comp, &mut colored, COLOR_YUV2RGB_NV21).unwrap();
        let mut jpeg_buf = Vector::<u8>::new();
        imencode_def(".jpg", &colored, &mut jpeg_buf).unwrap();
        
        let path = time.format("/media/mmc/records/detected/%Y-%m-%d_%H_%M_%S.jpg").to_string();
        let file = File::create(&path).unwrap();
        let mut writer = BufWriter::new(file);
        writer.write_all(jpeg_buf.as_slice()).unwrap();
        writer.into_inner().unwrap().sync_all().unwrap();

        let path = time.format("/media/mmc/records/detected/%Y-%m-%d_%H_%M_%S.fits").to_string();
        let image_description = ImageDescription {
            data_type: ImageType::UnsignedByte,
            dimensions: &[1080 + 540, 1920],
        };

        let mut fitsfile = FitsFile::create(&path)
            .with_custom_primary(&image_description)
            .overwrite()
            .open().unwrap();
        
        let hdu = fitsfile.primary_hdu().unwrap();

        hdu.write_image(&mut fitsfile, &frame).unwrap();
    }
}
