use crate::config::save_atomconf;
use crate::config::save_to_file;
use crate::config::AtomConfig;
use crate::gpio::*;
use crate::system;
use crate::AppState;
use axum::extract::{
    ws::{Message, WebSocket},
    State, WebSocketUpgrade,
};
use axum::response::IntoResponse;
use futures::SinkExt;
use futures::StreamExt;
use isvp_sys::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tokio::sync::watch;

type AppStateWs = State<(
    watch::Receiver<Vec<u8>>,
    Arc<Mutex<AppState>>,
    Arc<Mutex<AtomConfig>>,
    watch::Receiver<LogType>,
    Arc<Mutex<bool>>,
)>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LogType {
    Detection(String, String),
    None,
}

pub async fn handler(
    ws: WebSocketUpgrade,
    State((rx, app_state, atom_conf, log_rx, flag)): AppStateWs,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket: WebSocket| {
        handle_socket(socket, rx, app_state, atom_conf, log_rx, flag)
    })
}

pub async fn handle_socket(
    socket: WebSocket,
    mut rx: watch::Receiver<Vec<u8>>,
    app_state: Arc<Mutex<AppState>>,
    atom_conf: Arc<Mutex<AtomConfig>>,
    mut log_rx: watch::Receiver<LogType>,
    flag: Arc<Mutex<bool>>,
) {
    let (mut sender, mut receiver) = socket.split();
    let (time_tx, mut time_rx) = mpsc::channel(2);

    // To avoid deadlock.
    let app_state_message = {
        let app_state_tmp = match app_state.lock() {
            Ok(guard) => guard,
            Err(_) => {
                _ = sender.close();
                return;
            }
        };
        let app_state_json = serde_json::to_string(&app_state_tmp.clone()).unwrap();
        drop(app_state_tmp);
        Message::Text(format!(
            "{{\"type\":\"appstate\",\"payload\":{}}}",
            app_state_json
        ))
    };

    if sender.send(app_state_message).await.is_err() {
        return;
    }

    tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            let mut app_state_tmp = match app_state.lock() {
                Ok(guard) => guard,
                Err(_) => continue,
            };
            match msg {
                Message::Text(texta) => {
                    let text: Vec<&str> = texta.split(',').collect();
                    match text[0] {
                        // Measure round trip time.
                        "time" => {
                            let now = std::time::Instant::now();
                            drop(app_state_tmp);
                            if time_tx.send(now).await.is_err() {
                                break;
                            }
                        }
                        "mode" => {
                            if text.len() == 2 {
                                if text[1] == "day" {
                                    app_state_tmp.night_mode = false;
                                    drop(app_state_tmp);
                                    unsafe {
                                        IMP_ISP_Tuning_SetISPRunningMode(
                                            IMPISPRunningMode_IMPISP_RUNNING_MODE_DAY,
                                        );
                                    }
                                } else if text[1] == "night" {
                                    app_state_tmp.night_mode = true;
                                    drop(app_state_tmp);
                                    unsafe {
                                        IMP_ISP_Tuning_SetISPRunningMode(
                                            IMPISPRunningMode_IMPISP_RUNNING_MODE_NIGHT,
                                        );
                                    }
                                }
                            }
                        }
                        "ir" => {
                            if text.len() == 2 {
                                if text[1] == "on" {
                                    app_state_tmp.ircut_on = true;
                                    drop(app_state_tmp);
                                    ircut_on().unwrap();
                                } else if text[1] == "off" {
                                    app_state_tmp.ircut_on = false;
                                    drop(app_state_tmp);
                                    ircut_off().unwrap();
                                }
                            }
                        }
                        "led" => {
                            if text.len() == 2 {
                                if text[1] == "on" {
                                    app_state_tmp.led_on = true;
                                    drop(app_state_tmp);
                                } else if text[1] == "off" {
                                    app_state_tmp.led_on = false;
                                    drop(app_state_tmp);
                                }
                            }
                        }
                        "irled" => {
                            if text.len() == 2 {
                                if text[1] == "on" {
                                    app_state_tmp.irled_on = true;
                                    drop(app_state_tmp);
                                    irled_on().unwrap();
                                } else if text[1] == "off" {
                                    app_state_tmp.irled_on = false;
                                    drop(app_state_tmp);
                                    irled_off().unwrap();
                                }
                            }
                        }
                        "flip" => {
                            if text.len() == 3 {
                                if text[1] == "h" {
                                    if text[2] == "on" {
                                        app_state_tmp.flip.0 = true;
                                        drop(app_state_tmp);
                                        unsafe {
                                            IMP_ISP_Tuning_SetISPHflip(
                                                IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_ENABLE,
                                            );
                                        }
                                    } else if text[2] == "off" {
                                        app_state_tmp.flip.0 = false;
                                        drop(app_state_tmp);
                                        unsafe {
                                            IMP_ISP_Tuning_SetISPHflip(
                                                IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_DISABLE,
                                            );
                                        }
                                    }
                                } else if text[1] == "v" {
                                    if text[2] == "on" {
                                        app_state_tmp.flip.1 = true;
                                        drop(app_state_tmp);
                                        unsafe {
                                            IMP_ISP_Tuning_SetISPVflip(
                                                IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_ENABLE,
                                            );
                                        }
                                    } else if text[2] == "off" {
                                        app_state_tmp.flip.1 = false;
                                        drop(app_state_tmp);
                                        unsafe {
                                            IMP_ISP_Tuning_SetISPVflip(
                                                IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_DISABLE,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        "fps" => {
                            if text.len() == 2 {
                                let fps = text[1].parse().unwrap();
                                if matches!(fps, 5 | 10 | 15 | 20 | 25) {
                                    app_state_tmp.fps = fps;
                                    drop(app_state_tmp);
                                    unsafe {
                                        IMP_ISP_Tuning_SetSensorFPS(fps, 1);
                                    }
                                }
                            }
                        }
                        "proc" => {
                            if text.len() == 3 {
                                let v = text[2].parse().unwrap();
                                unsafe {
                                    match text[1] {
                                        "sat" => {
                                            app_state_tmp.saturation = v;
                                            drop(app_state_tmp);
                                            IMP_ISP_Tuning_SetSaturation(v);
                                        }
                                        "brt" => {
                                            app_state_tmp.brightness = v;
                                            drop(app_state_tmp);
                                            IMP_ISP_Tuning_SetBrightness(v);
                                        }
                                        "cnt" => {
                                            app_state_tmp.contrast = v;
                                            drop(app_state_tmp);
                                            IMP_ISP_Tuning_SetContrast(v);
                                        }
                                        "shrp" => {
                                            app_state_tmp.sharpness = v;
                                            drop(app_state_tmp);
                                            IMP_ISP_Tuning_SetSharpness(v);
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        "det" => {
                            if text.len() == 2 {
                                if text[1] == "on" {
                                    app_state_tmp.detect = true;
                                    drop(app_state_tmp);
                                } else if text[1] == "off" {
                                    app_state_tmp.detect = false;
                                    drop(app_state_tmp);
                                }
                            }
                        }
                        "tstmp" => {
                            if text.len() == 2 {
                                if text[1] == "on" {
                                    app_state_tmp.timestamp = true;
                                    drop(app_state_tmp);
                                } else if text[1] == "off" {
                                    app_state_tmp.timestamp = false;
                                    drop(app_state_tmp);
                                }
                            }
                        }
                        "save" => {
                            let app_state_clone = app_state_tmp.clone();
                            drop(app_state_tmp);
                            tokio::spawn(save_to_file(app_state_clone));
                        }
                        "net" => {
                            drop(app_state_tmp);
                            if text.len() == 4 {
                                let mut atom_conf_tmp = match atom_conf.lock() {
                                    Ok(guard) => guard,
                                    Err(_) => {
                                        continue;
                                    }
                                };
                                atom_conf_tmp.netconf.ap_mode = text[1] == "true";
                                atom_conf_tmp.netconf.ssid = text[2].to_string();
                                atom_conf_tmp.netconf.psk = text[3].to_string();
                                let atomconf_instance = atom_conf_tmp.clone();
                                tokio::spawn(save_atomconf(atomconf_instance));
                                drop(atom_conf_tmp);
                            }
                        }
                        "sync" => {
                            if text.len() == 4 {
                                app_state_tmp.timezone = text[3].parse().unwrap();
                                drop(app_state_tmp);
                                let new_time = timeval {
                                    tv_sec: text[1].parse().unwrap(),
                                    tv_usec: text[2].parse().unwrap(),
                                };
                                unsafe {
                                    let timezone_: *const timezone = std::ptr::null();

                                    let result = settimeofday(&new_time, timezone_);

                                    if result == 0 {
                                        println!("Set system time");
                                    } else {
                                        println!("Failed to set system time");
                                    }
                                }
                            }
                        }
                        "reboot" => {
                            drop(app_state_tmp);
                            tokio::spawn(system::reboot(flag));
                            break;
                        }
                        _ => {
                            println!("Unknwon command")
                        }
                    }
                }
                Message::Binary(buffer) => {
                    app_state_tmp.mask = buffer;
                    drop(app_state_tmp);
                }
                Message::Close(_) => break,
                _ => (),
            }
        }
    });

    tokio::spawn(async move {
        loop {
            let main = async {
                tokio::select! {
                    val = log_rx.changed() => {
                        if val.is_ok() {
                            let msg = match log_rx.borrow_and_update().clone() {
                                LogType::Detection(timestamp, mp4path) => {
                                    format!(
                                        "{{\"type\":\"detected\",\"payload\":{{\"timestamp\":\"{}\",\"saved_file\":\"{}\"}}}}",
                                        timestamp,
                                        mp4path
                                    )
                                },
                                _ => "".to_string(),
                            };
                            Message::Text(msg)
                        } else {
                            Message::Close(None)
                        }
                    }
                    val = rx.changed() => {
                        if val.is_ok() {
                            let img = rx.borrow_and_update().clone();
                            Message::Binary(img)
                        } else {
                            Message::Close(None)
                        }
                    }
                }
            };

            let message = tokio::select! {
                val = main => {
                    if matches!(val, Message::Close(None)) {
                        break;
                    }
                    val
                }
                val = time_rx.recv() => {
                    if let Some(time) = val {
                        let now = std::time::Instant::now();
                        let duration = now.duration_since(time);
                        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

                        let msg = format!(
                            "{{\"type\":\"time\",\"payload\":{{\"duration\":\"{}\", \"time\":\"{}\"}}}}",
                            duration.as_micros(),
                            time.as_millis()
                        );

                        Message::Text(msg)
                    } else {
                        break;
                    }
                }
            };

            if sender.send(message).await.is_err() {
                break;
            }
        }
        _ = sender.close();
    });
}
