#![allow(dead_code)]
#![feature(div_duration)]

use crate::config::load_from_file;
use crate::config::save_to_file;
use crate::detection::*;
use crate::gpio::*;
use crate::imp::*;
use crate::isp::log_all_value;
use crate::osd::*;
use crate::record::*;
use crate::websocket::*;
use axum::routing::get;
use axum::Router;
use serde::{Deserialize, Serialize};
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::{net::SocketAddr, path::PathBuf};
use tokio::sync::watch;
use tower_http::services::ServeDir;
use crate::download::download_file;

mod config;
mod detection;
mod font;
mod gpio;
mod imp;
mod isp;
mod osd;
mod record;
mod websocket;
mod download;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DetectionConfig {
    pub std_weight: f64,
    pub threshold: f64,
    pub max_roi_size: usize,
    pub length_threshold: u32,
    pub distance_threshold: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppState {
    pub mask: Vec<u8>, // len: 18*32
    pub detect: bool,
    pub detection_config: DetectionConfig,
    pub timestamp: bool,
    pub night_mode: bool,
    pub ircut_on: bool,
    pub led_on: bool,
    pub irled_on: bool,
    pub flip: (bool, bool), // Horizontal, Vertical
    pub brightness: u8,
    pub contrast: u8,
    pub sharpness: u8,
    pub saturation: u8,
    pub logs: Vec<LogType>,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let app_state = match load_from_file().await {
        Ok(p) => p,
        Err(_) => {
            let app_state = AppState {
                mask: vec![0u8; 18 * 32],
                detect: false,
                detection_config: DetectionConfig {
                    max_roi_size: 20,
                    std_weight: 2.,
                    threshold: 30.,
                    length_threshold: 10,
                    distance_threshold: 1.732,
                },
                timestamp: false,
                night_mode: false,
                ircut_on: false,
                led_on: true,
                irled_on: false,
                flip: (false, false),
                brightness: 128,
                contrast: 128,
                sharpness: 128,
                saturation: 128,
                logs: vec![],
            };
            save_to_file(app_state.clone()).await.unwrap();
            app_state
        }
    };
    let app_state_clone = app_state.clone();

    let (tx, rx) = watch::channel(vec![]);
    let (logtx, logrx) = watch::channel(LogType::None);
    let app_state_common = Arc::new(Mutex::new(app_state));
    let (detected_tx, detected_rx) = mpsc::channel();

    gpio_init().unwrap();

    if app_state_clone.ircut_on {
        ircut_on().unwrap();
    }

    if app_state_clone.irled_on {
        irled_on().unwrap();
    }

    let app_state_common_instance = app_state_common.clone();
    thread::spawn(move || {
        let mut blue_on = false;
        loop {
            let app_state = app_state_common_instance.lock().unwrap();
            if app_state.led_on {
                if !app_state.detect {
                    if blue_on {
                        led_off(LEDType::Blue).unwrap();
                        led_on(LEDType::Orange).unwrap();
                    } else {
                        led_off(LEDType::Orange).unwrap();
                        led_on(LEDType::Blue).unwrap();
                    }

                    blue_on = !blue_on;
                } else {
                    led_on(LEDType::Blue).unwrap();
                    led_off(LEDType::Orange).unwrap();
                }
            } else {
                led_off(LEDType::Blue).unwrap();
                led_off(LEDType::Orange).unwrap();
            }
            drop(app_state);

            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });

    let app_state_common_instance1 = app_state_common.clone();
    let app_state_common_instance2 = app_state_common.clone();

    thread::spawn(|| unsafe {
        imp_init(app_state_clone);
        log_all_value();
        imp_framesource_init();
        imp_encoder_init();
        let (grp_num, font_handle) = imp_osd_bind();
        imp_jpeg_init();
        imp_avc_init();
        imp_framesource_start();
        init();

        thread::spawn(move || {
            imp_osd_start(grp_num, font_handle, app_state_common_instance1);
        });

        mp4save_loops(detected_rx);

        thread::spawn(|| {
            jpeg_start(tx);
        });

        start(app_state_common_instance2, detected_tx, logtx);
    });

    let assets_dir = PathBuf::from("/media/mmc/assets/");

    let app = Router::new()
        .fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
        .route("/ws", get(handler))
        .route("/download", get(download_file))
        .with_state((rx, app_state_common, logrx));

    // run it with hyper
    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
