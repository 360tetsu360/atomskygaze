#![allow(dead_code)]

use crate::gpio::*;
use crate::imp::*;
use crate::isp::*;
use crate::record::*;
use crate::websocket::*;
use axum::routing::get;
use axum::Router;
use std::thread;
use std::{net::SocketAddr, path::PathBuf};
use tokio::sync::{mpsc, watch};
use tower_http::{
    services::ServeDir,
};

mod detection;
mod gpio;
mod imp;
mod isp;
mod record;
mod websocket;

#[tokio::main]
async fn main() {
    env_logger::init();

    println!("HelloA!!!");

    let (_tx, rx) = watch::channel(vec![]);
    let (mtx, _mrx) = mpsc::channel(2);

    gpio_init().unwrap();

    thread::spawn(|| {
        unsafe {
            imp_init();
            log_all_value();
            imp_framesource_init();
            imp_encoder_init();
            //imp_jpeg_init();
            imp_avc_init();
            imp_framesource_start();
            //IMP_ISP_Tuning_SetISPBypass(IMPISPTuningOpsMode_IMPISP_TUNING_OPS_MODE_DISABLE);
            //thread::spawn(|| {
            //    get_timelapse_h264_stream();
            //});
            get_h264_stream();
            //jpeg_start(tx);
            //init();
            //start(tx, mrx);
        }
    });

    let assets_dir = PathBuf::from("/media/mmc/assets/");

    let app = Router::new()
        .fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
        .route("/ws", get(handler))
        .with_state((rx, mtx));

    // run it with hyper
    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
