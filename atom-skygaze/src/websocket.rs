use crate::config::save_to_file;
use crate::config::NetworkConfig;
use crate::gpio::*;
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
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::watch;

type AppStateWs = State<(
    watch::Receiver<Vec<u8>>,
    Arc<Mutex<AppState>>,
    watch::Receiver<LogType>,
)>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LogType {
    Detection(String, String),
    None,
}

pub async fn handler(
    ws: WebSocketUpgrade,
    State((rx, app_state, log_rx)): AppStateWs,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket: WebSocket| handle_socket(socket, rx, app_state, log_rx))
}

pub async fn handle_socket(
    socket: WebSocket,
    mut rx: watch::Receiver<Vec<u8>>,
    app_state: Arc<Mutex<AppState>>,
    mut log_rx: watch::Receiver<LogType>,
) {
    let (mut sender, mut receiver) = socket.split();
    let app_state_json = serde_json::to_string(&(*app_state.lock().unwrap()).clone()).unwrap();
    let app_state_message = Message::Text(format!(
        "{{\"type\":\"appstate\",\"payload\":{}}}",
        app_state_json
    ));

    if sender.send(app_state_message).await.is_err() {
        return;
    }

    tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(texta) => {
                    let text: Vec<&str> = texta.split(',').collect();
                    match text[0] {
                        "mode" => {
                            if text.len() == 2 {
                                if text[1] == "day" {
                                    unsafe {
                                        IMP_ISP_Tuning_SetISPRunningMode(
                                            IMPISPRunningMode_IMPISP_RUNNING_MODE_DAY,
                                        );
                                    }
                                    app_state.lock().unwrap().night_mode = false;
                                } else if text[1] == "night" {
                                    unsafe {
                                        IMP_ISP_Tuning_SetISPRunningMode(
                                            IMPISPRunningMode_IMPISP_RUNNING_MODE_NIGHT,
                                        );
                                    }
                                    app_state.lock().unwrap().night_mode = true;
                                }
                            }
                        }
                        "ir" => {
                            if text.len() == 2 {
                                if text[1] == "on" {
                                    ircut_on().unwrap();
                                    app_state.lock().unwrap().ircut_on = true;
                                } else if text[1] == "off" {
                                    ircut_off().unwrap();
                                    app_state.lock().unwrap().ircut_on = false;
                                }
                            }
                        }
                        "led" => {
                            if text.len() == 2 {
                                if text[1] == "on" {
                                    app_state.lock().unwrap().led_on = true;
                                } else if text[1] == "off" {
                                    app_state.lock().unwrap().led_on = false;
                                }
                            }
                        }
                        "irled" => {
                            if text.len() == 2 {
                                if text[1] == "on" {
                                    irled_on().unwrap();
                                    app_state.lock().unwrap().irled_on = true;
                                } else if text[1] == "off" {
                                    irled_off().unwrap();
                                    app_state.lock().unwrap().irled_on = false;
                                }
                            }
                        }
                        "flip" => {
                            if text.len() == 3 {
                                if text[1] == "h" {
                                    if text[2] == "on" {
                                        unsafe {
                                            IMP_ISP_Tuning_SetISPHflip(
                                                IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_ENABLE,
                                            );
                                        }
                                        app_state.lock().unwrap().flip.0 = true;
                                    } else if text[2] == "off" {
                                        unsafe {
                                            IMP_ISP_Tuning_SetISPHflip(
                                                IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_DISABLE,
                                            );
                                        }
                                        app_state.lock().unwrap().flip.0 = false;
                                    }
                                } else if text[1] == "v" {
                                    if text[2] == "on" {
                                        unsafe {
                                            IMP_ISP_Tuning_SetISPVflip(
                                                IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_ENABLE,
                                            );
                                        }
                                        app_state.lock().unwrap().flip.1 = true;
                                    } else if text[2] == "off" {
                                        unsafe {
                                            IMP_ISP_Tuning_SetISPVflip(
                                                IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_DISABLE,
                                            );
                                        }
                                        app_state.lock().unwrap().flip.1 = false;
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
                                            IMP_ISP_Tuning_SetSaturation(v);
                                            app_state.lock().unwrap().saturation = v;
                                        }
                                        "brt" => {
                                            IMP_ISP_Tuning_SetBrightness(v);
                                            app_state.lock().unwrap().brightness = v;
                                        }
                                        "cnt" => {
                                            IMP_ISP_Tuning_SetContrast(v);
                                            app_state.lock().unwrap().contrast = v;
                                        }
                                        "shrp" => {
                                            IMP_ISP_Tuning_SetSharpness(v);
                                            app_state.lock().unwrap().sharpness = v;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        "det" => {
                            if text.len() == 2 {
                                if text[1] == "on" {
                                    app_state.lock().unwrap().detect = true;
                                } else if text[1] == "off" {
                                    app_state.lock().unwrap().detect = false;
                                }
                            }
                        }
                        "tstmp" => {
                            if text.len() == 2 {
                                if text[1] == "on" {
                                    app_state.lock().unwrap().timestamp = true;
                                } else if text[1] == "off" {
                                    app_state.lock().unwrap().timestamp = false;
                                }
                            }
                        }
                        "save" => {
                            let app_state_clone = (*app_state.lock().unwrap()).clone();
                            tokio::spawn(save_to_file(app_state_clone));
                        }
                        "netconf" => {
                            if text.len() == 4 {
                                let mut netconf = NetworkConfig {
                                    ap_mode: false,
                                    ssid: "".to_string(),
                                    psk: "".to_string(),
                                };

                                if text[1] == "on" {
                                    netconf.ap_mode = true;
                                } else if text[1] == "off" {
                                    netconf.ap_mode = false;
                                } else {
                                    return;
                                }

                                netconf.ssid = text[2].to_string();
                                netconf.psk = text[3].to_string();

                                //tokio::spawn(save_netconf(netconf));
                            }
                        }
                        "reboot" => {
                            unsafe {
                                SU_Base_Reboot();
                            }
                        }
                        _ => {}
                    }
                }
                Message::Binary(buffer) => {
                    app_state.lock().unwrap().mask = buffer;
                }
                Message::Close(_) => break,
                _ => (),
            }
        }
    });

    tokio::spawn(async move {
        loop {
            let message = tokio::select! {
                val = log_rx.changed() => {
                    if val.is_ok() {
                        let msg = match log_rx.borrow_and_update().clone() {
                            LogType::Detection(timestamp, mp4path) => {
                                format!("{{\"type\":\"detected\",\"payload\":{{\"timestamp\":\"{}\",\"saved_file\":\"{}\"}}}}", timestamp, mp4path)
                            },
                            _ => "".to_string(),
                        };
                        Message::Text(msg)
                    } else {
                        break;
                    }
                }
                val = rx.changed() => {
                    if val.is_ok() {
                        let img = rx.borrow_and_update().clone();
                        Message::Binary(img)
                    } else {
                        break;
                    }
                }
            };

            if sender.send(message).await.is_err() {
                break;
            }
        }
    });
}
