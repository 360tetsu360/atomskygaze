use crate::gpio::*;
use axum::extract::{
    ws::{Message, WebSocket},
    State, WebSocketUpgrade,
};
use axum::response::IntoResponse;
use chrono::*;
use futures::SinkExt;
use futures::StreamExt;
use isvp_sys::*;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::{mpsc, watch};

type AppState = State<(
    watch::Receiver<Vec<u8>>,
    mpsc::Sender<Vec<u8>>,
    Arc<Mutex<bool>>,
    Arc<Mutex<Vec<(DateTime<FixedOffset>, DateTime<FixedOffset>)>>>,
)>;

pub async fn handler(
    ws: WebSocketUpgrade,
    State((rx, mask_sender, detecting, detected)): AppState,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket: WebSocket| {
        handle_socket(socket, rx, mask_sender, detecting, detected)
    })
}

pub async fn handle_socket(
    socket: WebSocket,
    mut rx: watch::Receiver<Vec<u8>>,
    mask_sender: mpsc::Sender<Vec<u8>>,
    detecting: Arc<Mutex<bool>>,
    detected: Arc<Mutex<Vec<(DateTime<FixedOffset>, DateTime<FixedOffset>)>>>,
) {
    let (mut sender, mut receiver) = socket.split();
    tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            dbg!();
            match msg {
                Message::Text(texta) => {
                    dbg!(&texta);
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
                                } else if text[1] == "night" {
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
                                    ircut_on().unwrap();
                                } else if text[1] == "off" {
                                    ircut_off().unwrap();
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
                                    } else if text[2] == "off" {
                                        unsafe {
                                            IMP_ISP_Tuning_SetISPHflip(
                                                IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_DISABLE,
                                            );
                                        }
                                    }
                                } else if text[1] == "v" {
                                    if text[2] == "on" {
                                        unsafe {
                                            IMP_ISP_Tuning_SetISPVflip(
                                                IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_ENABLE,
                                            );
                                        }
                                    } else if text[2] == "off" {
                                        unsafe {
                                            IMP_ISP_Tuning_SetISPVflip(
                                                IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_DISABLE,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        "ae" => {
                            if text.len() == 3 {
                                let mut aeattr = IMPISPAEAttr {
                                    AeFreezenEn: 0,
                                    AeItManualEn: 0,
                                    AeIt: 0,
                                    AeAGainManualEn: 0,
                                    AeAGain: 0,
                                    AeDGainManualEn: 0,
                                    AeDGain: 0,
                                    AeIspDGainManualEn: 0,
                                    AeIspDGain: 0,
                                    AeWdrShortFreezenEn: 0,
                                    AeWdrShortItManualEn: 0,
                                    AeWdrShortIt: 0,
                                    AeWdrShortAGainManualEn: 0,
                                    AeWdrShortAGain: 0,
                                    AeWdrShortDGainManualEn: 0,
                                    AeWdrShortDGain: 0,
                                    AeWdrShortIspDGainManualEn: 0,
                                    AeWdrShortIspDGain: 0,
                                };

                                unsafe {
                                    IMP_ISP_Tuning_GetAeAttr(&mut aeattr);
                                }

                                match text[1] {
                                    "freeze" => {
                                        if text[2] == "on" {
                                            aeattr.AeFreezenEn =
                                                IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_ENABLE;
                                        } else if text[2] == "off" {
                                            aeattr.AeFreezenEn =
                                                IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_DISABLE;
                                        }
                                    }
                                    "expr-en" => {
                                        if text[2] == "on" {
                                            aeattr.AeItManualEn =
                                                IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_ENABLE;
                                        } else if text[2] == "off" {
                                            aeattr.AeItManualEn =
                                                IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_DISABLE;
                                        }
                                    }
                                    "again-en" => {
                                        if text[2] == "on" {
                                            aeattr.AeAGainManualEn =
                                                IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_ENABLE;
                                        } else if text[2] == "off" {
                                            aeattr.AeAGainManualEn =
                                                IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_DISABLE;
                                        }
                                    }
                                    "dgain-en" => {
                                        if text[2] == "on" {
                                            aeattr.AeDGainManualEn =
                                                IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_ENABLE;
                                        } else if text[2] == "off" {
                                            aeattr.AeDGainManualEn =
                                                IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_DISABLE;
                                        }
                                    }
                                    "ispgain-en" => {
                                        if text[2] == "on" {
                                            aeattr.AeIspDGainManualEn =
                                                IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_ENABLE;
                                        } else if text[2] == "off" {
                                            aeattr.AeIspDGain =
                                                IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_DISABLE;
                                        }
                                    }
                                    "expr" => {
                                        aeattr.AeIt = text[2].parse().unwrap();
                                    }
                                    "again" => {
                                        aeattr.AeAGain = text[2].parse().unwrap();
                                    }
                                    "dgain" => {
                                        aeattr.AeDGain = text[2].parse().unwrap();
                                    }
                                    "ispgain" => {
                                        aeattr.AeIspDGain = text[2].parse().unwrap();
                                    }
                                    _ => {}
                                }

                                unsafe {
                                    IMP_ISP_Tuning_SetAeAttr(&mut aeattr);
                                }
                            }
                        }
                        "wdr" => {
                            if text.len() == 2 {
                                if text[1] == "on" {
                                    unsafe {
                                        IMP_ISP_WDR_ENABLE(
                                            IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_ENABLE,
                                        );
                                    }
                                } else if text[1] == "off" {
                                    unsafe {
                                        IMP_ISP_WDR_ENABLE(
                                            IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_DISABLE,
                                        );
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
                                        }
                                        "brt" => {
                                            IMP_ISP_Tuning_SetBrightness(v);
                                        }
                                        "cnt" => {
                                            IMP_ISP_Tuning_SetContrast(v);
                                        }
                                        "shrp" => {
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
                                    *detecting.lock().unwrap() = true;
                                } else if text[1] == "off" {
                                    *detecting.lock().unwrap() = false;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Message::Binary(buffer) => {
                    mask_sender.send(buffer).await.unwrap();
                }
                Message::Close(_) => break,
                _ => (),
            }
        }
    });

    tokio::spawn(async move {
        while (rx.changed().await).is_ok() {
            let img = rx.borrow_and_update().clone();
            if sender.send(Message::Binary(img)).await.is_err() {
                break;
            }
        }
    });
}
