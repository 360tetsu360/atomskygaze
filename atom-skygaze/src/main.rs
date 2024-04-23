#![allow(dead_code)]
#![feature(div_duration)]

use crate::detection::*;
use crate::gpio::*;
use crate::imp::*;
use crate::isp::log_all_value;
use crate::osd::*;
use crate::record::*;
use crate::trim::trim_loop;
use crate::websocket::*;
use axum::routing::get;
use axum::Router;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::{net::SocketAddr, path::PathBuf};
use tokio::sync::{mpsc, watch};
use tower_http::services::ServeDir;

mod detection;
mod font;
mod gpio;
mod imp;
mod isp;
mod osd;
mod record;
mod trim;
mod websocket;

#[tokio::main]
async fn main() {
    env_logger::init();

    let (tx, rx) = watch::channel(vec![]);
    let (mtx, mrx) = mpsc::channel(2);
    let is_detecting = Arc::new(Mutex::new(false));
    let detected_msg = Arc::new(Mutex::new(vec![]));

    gpio_init().unwrap();

    let is_detecting_instance = is_detecting.clone();
    thread::spawn(move || {
        let mut blue_on = false;
        loop {
            if !*is_detecting_instance.lock().unwrap() {
                if blue_on {
                    led_off(LEDType::Blue).unwrap();
                    led_on(LEDType::Orange).unwrap();
                } else {
                    led_off(LEDType::Orange).unwrap();
                    led_on(LEDType::Blue).unwrap();
                }

                blue_on = !blue_on;
            }

            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });

    let detected_msg_instance = detected_msg.clone();
    thread::spawn(move || {
        trim_loop(detected_msg_instance);
    });

    let detected_msg_instance = detected_msg.clone();
    let is_detecting_instance = is_detecting.clone();

    thread::spawn(|| unsafe {
        imp_init();
        log_all_value();
        imp_framesource_init();
        imp_encoder_init();
        let (grp_num, font_handle) = imp_osd_bind();
        imp_jpeg_init();
        imp_avc_init();
        imp_framesource_start();
        init();

        thread::spawn(move || {
            imp_osd_start(grp_num, font_handle);
        });
        mp4save_loops();
        thread::spawn(|| {
            jpeg_start(tx);
        });

        start(mrx, is_detecting_instance, detected_msg_instance);
    });

    let assets_dir = PathBuf::from("/media/mmc/assets/");

    let app = Router::new()
        .fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
        .route("/ws", get(handler))
        .with_state((rx, mtx, is_detecting, detected_msg));

    // run it with hyper
    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
