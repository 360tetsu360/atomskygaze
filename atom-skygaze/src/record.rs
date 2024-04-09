use chrono::{Datelike, Local, Timelike};
use isvp_sys::*;
use log::{error, info};
use minimp4::Mp4Muxer;
use std::fs::File;
use std::path::Path;

const REGULAR_FPS: u32 = 60;
const TIMELAPSE_FPS: u32 = 25;

pub unsafe fn get_h264_stream() -> bool {
    if IMP_Encoder_StartRecvPic(3) < 0 {
        error!("IMP_Encoder_StartRecvPic failed");
        return false;
    }

    loop {
        let current_time = Local::now();
        let next_minute = (current_time.minute() + 1) % 60;
        let target_time = current_time
            .with_minute(next_minute)
            .unwrap()
            .with_second(0)
            .unwrap();

        let dir_day = format!(
            "/media/mmc/records/regular/{:04}{:02}{:02}/",
            current_time.year(),
            current_time.month(),
            current_time.day()
        );

        let dir = Path::new(&dir_day);
        if !dir.is_dir() {
            std::fs::create_dir(dir).unwrap();
        }

        let hour_dir = dir.join(format!("{:02}", current_time.hour()));
        if !hour_dir.is_dir() {
            std::fs::create_dir(&hour_dir).unwrap();
        }

        let mp4path = format!(
            "/media/mmc/records/regular/{:04}{:02}{:02}/{:02}/{:02}.mp4",
            current_time.year(),
            current_time.month(),
            current_time.day(),
            current_time.hour(),
            current_time.minute()
        );

        let mp4title = format!(
            "{:04}-{:02}-{:02}-{:02}-{:02}",
            current_time.year(),
            current_time.month(),
            current_time.day(),
            current_time.hour(),
            current_time.minute()
        );

        let mut mp4muxer = Mp4Muxer::new(File::create(mp4path).unwrap());
        mp4muxer.init_video(1920, 1080, false, &mp4title);

        loop {
            let current_time = Local::now();
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
                seq: 0,
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
                        mp4muxer.write_video_with_fps(src1, REGULAR_FPS);
                        let src2 = std::slice::from_raw_parts(
                            stream.virAddr as *const u8,
                            (pack.length - rem_size) as usize,
                        );
                        mp4muxer.write_video_with_fps(src2, REGULAR_FPS);
                    } else {
                        let src = std::slice::from_raw_parts(
                            (stream.virAddr + pack.offset) as *const u8,
                            pack.length as usize,
                        );
                        mp4muxer.write_video_with_fps(src, REGULAR_FPS);
                    }
                }
            }

            if IMP_Encoder_ReleaseStream(3, &mut stream) < 0 {
                error!("IMP_Encoder_ReleaseStream failed");
                return false;
            }
        }

        mp4muxer.close();
        info!(
            "Saved regular {:04}{:02}{:02}{:02}{:02}",
            current_time.year(),
            current_time.month(),
            current_time.day(),
            current_time.hour(),
            current_time.minute()
        );
    }

    true
}

pub unsafe fn get_timelapse_h264_stream() -> bool {
    if IMP_Encoder_StartRecvPic(4) < 0 {
        error!("IMP_Encoder_StartRecvPic failed");
        return false;
    }

    loop {
        let current_time = Local::now();
        let next_hour = (current_time.hour() + 1) % 60;
        let target_time = current_time
            .with_minute(next_hour)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap();

        let dir_day = format!(
            "/media/mmc/records/timelapse/{:04}{:02}{:02}/",
            current_time.year(),
            current_time.month(),
            current_time.day()
        );

        let dir = Path::new(&dir_day);
        if !dir.is_dir() {
            std::fs::create_dir(dir).unwrap();
        }

        let mp4path = format!(
            "/media/mmc/records/timelapse/{:04}{:02}{:02}/{:02}.mp4",
            current_time.year(),
            current_time.month(),
            current_time.day(),
            current_time.hour(),
        );

        let mp4title = format!(
            "{:04}-{:02}-{:02}-{:02}",
            current_time.year(),
            current_time.month(),
            current_time.day(),
            current_time.hour(),
        );

        let mut mp4muxer = Mp4Muxer::new(File::create(mp4path).unwrap());
        mp4muxer.init_video(1920, 1080, false, &mp4title);

        loop {
            let current_time = Local::now();
            if current_time >= target_time {
                break;
            }

            if IMP_Encoder_PollingStream(4, 1000) < 0 {
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
            };

            if IMP_Encoder_GetStream(4, &mut stream, false) < 0 {
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
                        mp4muxer.write_video_with_fps(src1, TIMELAPSE_FPS);
                        let src2 = std::slice::from_raw_parts(
                            stream.virAddr as *const u8,
                            (pack.length - rem_size) as usize,
                        );
                        mp4muxer.write_video_with_fps(src2, TIMELAPSE_FPS);
                    } else {
                        let src = std::slice::from_raw_parts(
                            (stream.virAddr + pack.offset) as *const u8,
                            pack.length as usize,
                        );
                        mp4muxer.write_video_with_fps(src, TIMELAPSE_FPS);
                    }
                }
            }

            if IMP_Encoder_ReleaseStream(4, &mut stream) < 0 {
                error!("IMP_Encoder_ReleaseStream failed");
                return false;
            }
        }

        mp4muxer.close();
        info!(
            "Saved timelapse {:04}{:02}{:02}{:02}",
            current_time.year(),
            current_time.month(),
            current_time.day(),
            current_time.hour(),
        );
    }

    true
}
