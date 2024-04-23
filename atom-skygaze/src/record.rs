use chrono::{Datelike, Duration, Local, Timelike};
use isvp_sys::*;
use log::{error, info};
use minimp4::Mp4Muxer;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use std::sync::mpsc;
use std::thread;

const REGULAR_FPS: u32 = 25;

pub fn mp4save_loops() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || unsafe { get_h264_stream(tx) });

    thread::spawn(move || sync_file(rx));
}

unsafe fn get_h264_stream(tx: mpsc::Sender<File>) -> bool {
    if IMP_Encoder_StartRecvPic(3) < 0 {
        error!("IMP_Encoder_StartRecvPic failed");
        return false;
    }

    loop {
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

        let mut last_timestamp = -1;

        loop {
            let current_time = Local::now() + Duration::hours(9);
            if current_time >= target_time {
                break;
            }

            if IMP_Encoder_PollingStream(3, 1000) < 0 {
                error!("IMP_Encoder_PollingStream failed");
                return false;
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
                let fps = REGULAR_FPS;

                last_timestamp = pack.timestamp;

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
    for file in rx {
        file.sync_all().unwrap();
        println!("Saved mp4");
    }
}
