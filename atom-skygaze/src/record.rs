use crate::AppState;
use chrono::DateTime;
use chrono::FixedOffset;
use chrono::{Datelike, Duration, Local, Timelike};
use isvp_sys::*;
use log::error;
use minimp4::Mp4Muxer;
use std::collections::VecDeque;
use std::fs::File;
use std::path::Path;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

pub fn mp4save_loops(
    detected_rx: mpsc::Receiver<Option<DateTime<FixedOffset>>>,
    app_state: Arc<Mutex<AppState>>,
    flag: Arc<Mutex<bool>>,
) {
    let (tx, rx) = mpsc::channel();

    thread::Builder::new()
        .name("h264_loop".to_string())
        .spawn(move || unsafe { get_h264_stream(tx, detected_rx, app_state, flag) })
        .unwrap();

    thread::Builder::new()
        .name("filesave_loop".to_string())
        .spawn(move || sync_file(rx))
        .unwrap();
}

unsafe fn get_h264_stream(
    tx: mpsc::Sender<File>,
    detected_rx: mpsc::Receiver<Option<DateTime<FixedOffset>>>,
    app_state: Arc<Mutex<AppState>>,
    flag: Arc<Mutex<bool>>,
) -> bool {
    let mut queue = VecDeque::<Vec<u8>>::new();
    let mut is_detecting = false;
    let mut detect_video = None;

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

            if let Ok(det) = detected_rx.try_recv() {
                if det.is_some() && !is_detecting {
                    let time = det.unwrap();
                    let mp4path = time
                        .format("/media/mmc/records/detected/%Y%m%d%H%M%S.mp4")
                        .to_string();

                    let mp4title = time.format("Detected %Y-%m-%d %H:%M:%S").to_string();
                    let file = File::create(mp4path).unwrap();
                    let mut det_mp4muxer = Mp4Muxer::new(file);
                    det_mp4muxer.init_video(1920, 1080, true, &mp4title);

                    for frame in queue.iter() {
                        det_mp4muxer.write_video_with_fps(frame, fps);
                    }
                    detect_video = Some(det_mp4muxer);
                    is_detecting = true;
                }

                if det.is_none() && is_detecting {
                    let file = detect_video.take().unwrap().close();
                    tx.send(file).unwrap();
                    is_detecting = false;
                }
            }

            let current_time = Local::now() + Duration::hours(9);
            if current_time >= target_time {
                break;
            }

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

                        if is_detecting {
                            detect_video
                                .as_mut()
                                .unwrap()
                                .write_video_with_fps(src1, fps);
                            detect_video
                                .as_mut()
                                .unwrap()
                                .write_video_with_fps(src2, fps);
                        } else {
                            let buff: Vec<u8> = [src1, src2].concat();
                            queue.push_back(buff);
                        }
                    } else {
                        let src = std::slice::from_raw_parts(
                            (stream.virAddr + pack.offset) as *const u8,
                            pack.length as usize,
                        );
                        mp4muxer.write_video_with_fps(src, fps);

                        if is_detecting {
                            detect_video
                                .as_mut()
                                .unwrap()
                                .write_video_with_fps(src, fps);
                        } else {
                            let buff = src.to_vec();
                            queue.push_back(buff);
                        }
                    }
                }
            }

            if IMP_Encoder_ReleaseStream(3, &mut stream) < 0 {
                error!("IMP_Encoder_ReleaseStream failed");
                return false;
            }

            while queue.len() > 76 {
                queue.pop_front();
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
