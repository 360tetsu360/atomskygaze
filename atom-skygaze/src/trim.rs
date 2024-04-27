use chrono::*;
use log::error;
use std::fs::remove_file;
use std::fs::File;
use std::io::BufWriter;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex;

type DetectionReceiver = Arc<Mutex<Vec<(DateTime<FixedOffset>, DateTime<FixedOffset>)>>>;

pub fn trim_loop(msg_rx: DetectionReceiver) {
    let mut unprocessed_msg = vec![];
    loop {
        let mut msg_buff = msg_rx.lock().unwrap();
        while let Some((start, end)) = (*msg_buff).pop() {
            println!("{} {}", start, end);
            let output_path = start
                .format("/media/mmc/records/detected/%Y-%m-%d_%H_%M_%S.mp4")
                .to_string();

            if let Err(e) = trim_video(start, end, &output_path) {
                if e.kind() == ErrorKind::NotFound {
                    unprocessed_msg.push((start, end));
                } else {
                    error!("ffmpeg failed {}", e);
                }
            }
        }
        drop(msg_buff);

        let mut tmp = vec![];
        while let Some((start, end)) = unprocessed_msg.pop() {
            let output_path = start
                .format("/media/mmc/records/detected/%Y-%m-%d_%H_%M_%S.mp4")
                .to_string();
            if let Err(e) = trim_video(start, end, &output_path) {
                if e.kind() == ErrorKind::NotFound {
                    tmp.push((start, end));
                } else {
                    error!("ffmpeg failed {}", e);
                }
            }
        }

        unprocessed_msg = tmp;
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}

pub fn trim_video(
    start: DateTime<FixedOffset>,
    end: DateTime<FixedOffset>,
    output: &str,
) -> Result<()> {
    let start_file = start
        .format("/media/mmc/records/regular/%Y%m%d/%H/%M.mp4")
        .to_string();

    let end_file = end
        .format("/media/mmc/records/regular/%Y%m%d/%H/%M.mp4")
        .to_string();

    let file_path_start = Path::new(&start_file);
    let file_path_end = Path::new(&end_file);

    let next_start_file = (start + Duration::minutes(1))
        .format("/media/mmc/records/regular/%Y%m%d/%H/%M.mp4")
        .to_string();

    let next_end_file = (end + Duration::minutes(1))
        .format("/media/mmc/records/regular/%Y%m%d/%H/%M.mp4")
        .to_string();

    let next_file_path_start = Path::new(&next_start_file);
    let next_file_path_end = Path::new(&next_end_file);

    if !file_path_start.exists()
        || !file_path_end.exists()
        || !next_file_path_start.exists()
        || !next_file_path_end.exists()
    {
        return Err(Error::new(ErrorKind::NotFound, "mp4 not found"));
    }

    if start_file == end_file {
        let ffmpeg = Command::new("ffmpeg")
            .arg("-i")
            .arg(start_file)
            .arg("-ss")
            .arg(&start.format("00:00:%S").to_string())
            .arg("-to")
            .arg(&end.format("00:00:%S").to_string())
            .arg("-c")
            .arg("copy")
            .arg(output)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        match ffmpeg {
            Ok(mut child) => {
                let status = child.wait().expect("Child process wasn't running");
                if status.success() {
                    println!("Video trimmed successfully.");
                } else {
                    println!("Failed to trim video. Exit code: {:?}", status.code());
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("FFMPEG Error code {:?}", status.code()),
                    ));
                }
            }
            Err(e) => {
                println!("Failed to start FFmpeg process: {}", e);
                return Err(Error::new(ErrorKind::Other, e.to_string()));
            }
        }
    } else if end.second() == 0 {
        let ffmpeg = Command::new("ffmpeg")
            .arg("-i")
            .arg(start_file)
            .arg("-ss")
            .arg(&start.format("00:00:%S").to_string())
            .arg("-c")
            .arg("copy")
            .arg(output)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        match ffmpeg {
            Ok(mut child) => {
                let status = child.wait().expect("Child process wasn't running");
                if status.success() {
                    println!("Video trimmed successfully.");
                } else {
                    println!("Failed to trim video. Exit code: {:?}", status.code());
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("FFMPEG Error code {:?}", status.code()),
                    ));
                }
            }
            Err(e) => {
                println!("Failed to start FFmpeg process: {}", e);
                return Err(Error::new(ErrorKind::Other, e.to_string()));
            }
        }
    } else {
        let tmp_trim_start = start
            .format("/media/mmc/tmp/trim-%Y%m%d%H%M%S.mp4")
            .to_string();

        let ffmpeg = Command::new("ffmpeg")
            .arg("-i")
            .arg(start_file)
            .arg("-ss")
            .arg(&start.format("00:00:%S").to_string())
            .arg("-c")
            .arg("copy")
            .arg(&tmp_trim_start)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        match ffmpeg {
            Ok(mut child) => {
                let status = child.wait().unwrap();
                if status.success() {
                    println!("Video trimmed successfully.");
                } else {
                    println!("Failed to trim video. Exit code: {:?}", status.code());
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("FFMPEG Error code {:?}", status.code()),
                    ));
                }
            }
            Err(e) => {
                println!("Failed to start FFmpeg process: {}", e);
                return Err(Error::new(ErrorKind::Other, e.to_string()));
            }
        }

        let tmp_trim_end = end
            .format("/media/mmc/tmp/trim-%Y%m%d%H%M%S.mp4")
            .to_string();
        let ffmpeg = Command::new("ffmpeg")
            .arg("-i")
            .arg(end_file)
            .arg("-ss")
            .arg("00:00:00")
            .arg("-to")
            .arg(&end.format("00:00:%S").to_string())
            .arg("-c")
            .arg("copy")
            .arg(&tmp_trim_end)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn();

        match ffmpeg {
            Ok(mut child) => {
                let status = child.wait().unwrap();
                if status.success() {
                    println!("Video trimmed successfully.");
                } else {
                    println!("Failed to trim video. Exit code: {:?}", status.code());
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("FFMPEG Error code {:?}", status.code()),
                    ));
                }
            }
            Err(e) => {
                println!("Failed to start FFmpeg process: {}", e);
                return Err(Error::new(ErrorKind::Other, e.to_string()));
            }
        }

        let concat_path = start
            .format("/media/mmc/concat_list-%Y%m%d%H%M%S.txt")
            .to_string();
        let mut concat_list = File::create(&concat_path)?;
        let mut writer = BufWriter::new(&mut concat_list);
        writeln!(writer, "file '{}'", tmp_trim_start)?;
        writeln!(writer, "file '{}'", tmp_trim_end)?;
        writer.flush()?;
        drop(writer);
        concat_list.sync_all()?;

        let ffmpeg = Command::new("ffmpeg")
            .arg("-i")
            .arg(&concat_path)
            .arg("-f")
            .arg("concat")
            .arg("-safe")
            .arg("0")
            .arg("-c")
            .arg("copy")
            .arg(output)
            .spawn();

        match ffmpeg {
            Ok(mut child) => {
                let status = child.wait().expect("Child process wasn't running");
                if status.success() {
                    println!("Video concat successfully.");
                } else {
                    println!("Failed to concat video. Exit code: {:?}", status.code());
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("FFMPEG Error code {:?}", status.code()),
                    ));
                }
            }
            Err(e) => {
                println!("Failed to start FFmpeg process: {}", e);
                return Err(Error::new(ErrorKind::Other, e.to_string()));
            }
        }

        remove_file(&tmp_trim_start).unwrap();
        remove_file(&tmp_trim_end).unwrap();
        remove_file(&concat_path).unwrap();
    }

    Ok(())
}
