use crate::gpio::*;
use axum::extract::{
    ws::{Message, WebSocket},
    State, WebSocketUpgrade,
};
use axum::response::IntoResponse;
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
)>;

pub async fn handler(
    ws: WebSocketUpgrade,
    State((rx, mask_sender, detecting)): AppState,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket: WebSocket| handle_socket(socket, rx, mask_sender, detecting))
}

pub async fn handle_socket(
    socket: WebSocket,
    mut rx: watch::Receiver<Vec<u8>>,
    mask_sender: mpsc::Sender<Vec<u8>>,
    detecting: Arc<Mutex<bool>>,
) {
    let (mut sender, mut receiver) = socket.split();
    tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            dbg!();
            match msg {
                Message::Text(texta) => {
                    dbg!(&texta);
                    let text: Vec<&str> = texta.split(',').collect();
                    if text[0] == "iron" {
                        ircut_on().unwrap();
                    } else if text[0] == "iroff" {
                        ircut_off().unwrap();
                    } else if text[0] == "flip0" {
                        unsafe {
                            IMP_ISP_Tuning_SetHVFLIP(0);
                        }
                    } else if text[0] == "flip1" {
                        unsafe {
                            IMP_ISP_Tuning_SetHVFLIP(1);
                        }
                    } else if text[0] == "flip2" {
                        unsafe {
                            IMP_ISP_Tuning_SetHVFLIP(2);
                        }
                    } else if text[0] == "flip3" {
                        unsafe {
                            IMP_ISP_Tuning_SetHVFLIP(3);
                        }
                    } else if text[0] == "flip4" {
                        unsafe {
                            IMP_ISP_Tuning_SetHVFLIP(4);
                        }
                    } else if text[0] == "gain" {
                        let v = text[2].parse().unwrap();
                        match text[1] {
                            "again" => unsafe {
                                IMP_ISP_Tuning_SetMaxAgain(v);
                            },
                            "dgain" => unsafe {
                                IMP_ISP_Tuning_SetMaxDgain(v);
                            },
                            _ => {}
                        }
                    } else if text[0] == "expr" {
                        let mut vexpr = IMPISPExpr {
                            s_attr: isp_core_expr_attr__bindgen_ty_1 {
                                mode: isp_core_expr_mode_ISP_CORE_EXPR_MODE_MANUAL,
                                unit: isp_core_expr_unit_ISP_CORE_EXPR_UNIT_US,
                                time: text[1].parse().unwrap(),
                            },
                        };
                        unsafe {
                            IMP_ISP_Tuning_SetExpr(&mut vexpr);
                        }
                    } else if text[0] == "proc" {
                        let v = text[2].parse().unwrap();
                        match text[1] {
                            "brt" => unsafe {
                                IMP_ISP_Tuning_SetBrightness(v);
                            },
                            "cont" => unsafe {
                                IMP_ISP_Tuning_SetContrast(v);
                            },
                            "shrp" => unsafe {
                                IMP_ISP_Tuning_SetSharpness(v);
                            },
                            "satu" => unsafe {
                                IMP_ISP_Tuning_SetSaturation(v);
                            },
                            _ => {}
                        }
                    } else if text[0] == "whba" {
                        println!("r:{}, b:{}, channel:{}", text[1], text[2], text[3]);
                        let mut wb_attr = isp_core_wb_attr {
                            mode: text[3].parse().unwrap(),
                            rgain: text[1].parse().unwrap(),
                            bgain: text[2].parse().unwrap(),
                        };
                        unsafe {
                            IMP_ISP_Tuning_SetWB(&mut wb_attr);
                        }
                    } else if text[0] == "rgb" {
                        let mut wb_attr = isp_core_rgb_coefft_wb_attr {
                            rgb_coefft_wb_r: text[1].parse().unwrap(),
                            rgb_coefft_wb_g: text[2].parse().unwrap(),
                            rgb_coefft_wb_b: text[3].parse().unwrap(),
                        };
                        unsafe {
                            IMP_ISP_Tuning_Awb_SetRgbCoefft(&mut wb_attr);
                        }
                    } else if text[0] == "gamma" {
                        let mut gamma = IMPISPGamma { gamma: [0u16; 129] };
                        unsafe {
                            IMP_ISP_Tuning_GetGamma(&mut gamma);
                            let last = gamma.gamma[text[2].parse::<usize>().unwrap()];
                            println!("{}", last);
                            if text[3].parse().unwrap() {
                                gamma.gamma[text[2].parse::<usize>().unwrap()] =
                                    text[1].parse().unwrap();
                                IMP_ISP_Tuning_SetGamma(&mut gamma);
                            }
                        }
                    } else if text[0] == "mode" {
                        unsafe {
                            if text[1] == "day" {
                                IMP_ISP_Tuning_SetISPRunningMode(
                                    IMPISPRunningMode_IMPISP_RUNNING_MODE_DAY,
                                );
                            } else if text[1] == "night" {
                                IMP_ISP_Tuning_SetISPRunningMode(
                                    IMPISPRunningMode_IMPISP_RUNNING_MODE_NIGHT,
                                );
                            }
                        }
                    } else if text[0] == "det" {
                        if text[1] == "on" {
                            *detecting.lock().unwrap() = true;
                        } else if text[1] == "off" {
                            *detecting.lock().unwrap() = false;
                        }
                    } else if text[0] == "log" {
                        let mut eva_attr = IMPISPEVAttr {
                            ev: 0,
                            expr_us: 0,
                            ev_log2: 0,
                            again: 0,
                            dgain: 0,
                            gain_log2: 0,
                        };

                        let mut wb_attr = isp_core_rgb_coefft_wb_attr {
                            rgb_coefft_wb_r: 0,
                            rgb_coefft_wb_g: 0,
                            rgb_coefft_wb_b: 0,
                        };

                        let mut awb_hist = IMPISPAWBHist {
                            awb_stat: isp_core_awb_sta_info {
                                r_gain: 0,
                                b_gain: 0,
                                awb_sum: 0,
                            },
                            awb_stats_mode: 0,
                            awb_whitelevel: 0,
                            awb_blacklevel: 0,
                            cr_ref_max: 0,
                            cr_ref_min: 0,
                            cb_ref_max: 0,
                            cb_ref_min: 0,
                            awb_stat_nodeh: 0,
                            awb_stat_nodev: 0,
                        };

                        let mut gamma = IMPISPGamma { gamma: [0u16; 129] };
                        let mut mctl = IMPISPModuleCtl { key: 0 };
                        unsafe {
                            IMP_ISP_Tuning_GetEVAttr(&mut eva_attr);
                            IMP_ISP_Tuning_Awb_GetRgbCoefft(&mut wb_attr);
                            IMP_ISP_Tuning_GetAwbHist(&mut awb_hist);
                            IMP_ISP_Tuning_GetGamma(&mut gamma);
                            IMP_ISP_Tuning_SetISPBypass(
                                IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_ENABLE,
                            );
                            IMP_ISP_Tuning_GetModuleControl(&mut mctl);
                            dbg!(mctl.__bindgen_anon_1.bitBypassGAMMA());
                            mctl.__bindgen_anon_1.set_bitBypassGAMMA(0);
                            IMP_ISP_Tuning_SetModuleControl(&mut mctl);
                        }

                        dbg!(eva_attr);
                        dbg!(wb_attr);
                        dbg!(awb_hist);
                        dbg!(gamma);
                        dbg!(unsafe { mctl.key });
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
