use crate::config::*;
use crate::detection::*;
use crate::download::download_file;
use crate::download::view_file;
use crate::gpio::*;
use crate::imp::*;
use crate::isp::log_all_value;
use crate::osd::*;
use crate::record::*;
use crate::watchdog::WatchdogManager;
use crate::websocket::*;
use crate::webstream::*;
use axum::routing::get;
use axum::Router;
use log::*;
use serde::{Deserialize, Serialize};
use simplelog::{CombinedLogger, SimpleLogger, WriteLogger};
use std::fs::OpenOptions;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::{net::SocketAddr, path::PathBuf};
use tokio::sync::watch;
use tower_http::services::ServeDir;

mod config;
mod detection;
mod download;
mod font;
mod gpio;
mod imp;
mod isp;
mod osd;
mod record;
mod system;
mod watchdog;
mod websocket;
mod webstream;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct DetectionTime {
    pub start: (u32, u32),
    pub end: (u32, u32),
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct DetectionConfig {
    pub threshold: f64,
    pub max_roi_size: usize,
    pub length_threshold: u32,
    pub distance_threshold: f32,
    pub use_time: bool,
    pub detection_time: DetectionTime,
    pub solve_field: bool,
    pub save_wcs: bool,
    pub draw_constellation: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppState {
    pub mask: Vec<u8>, // len: 18*32
    pub detect: bool,
    pub detection_config: DetectionConfig,
    pub timestamp: bool,
    pub timestamp_pos: u32,
    pub night_mode: bool,
    pub ircut_on: bool,
    pub led_on: bool,
    pub irled_on: bool,
    pub flip: (bool, bool), // Horizontal, Vertical
    pub fps: u32,           // 5,10,15,20,25
    pub brightness: u8,
    pub contrast: u8,
    pub sharpness: u8,
    pub saturation: u8,
    pub timezone: i32,
    pub cap: bool,
    pub logs: Vec<LogType>,
}

#[tokio::main]
async fn main() {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("/media/mmc/atomskygaze.log")
        .unwrap();

    CombinedLogger::init(vec![
        SimpleLogger::new(LevelFilter::Info, simplelog::Config::default()),
        WriteLogger::new(LevelFilter::Info, simplelog::Config::default(), file),
    ])
    .unwrap();

    log::info!("Initializing watchdog");
    let watchdog = match WatchdogManager::init(10) {
        Ok(wd) => wd,
        Err(err) => {
            log::error!("WatchdogManager::init failed with error : {}", err);
            panic!();
        }
    };

    log::info!("Initializing config");
    let app_state = match load_from_file().await {
        Ok(p) => {
            log::info!("Found config file");
            p
        }
        Err(err) => {
            log::debug!("Failed to open the config file {}", err);
            log::info!("Creating config file");
            let app_state = AppState {
                mask: vec![0u8; 18 * 32],
                detect: false,
                detection_config: DetectionConfig {
                    max_roi_size: 20,
                    threshold: 5.,
                    length_threshold: 10,
                    distance_threshold: 1.732,
                    use_time: false,
                    detection_time: DetectionTime {
                        start: (18, 0),
                        end: (6, 0),
                    },
                    solve_field: false,
                    save_wcs: false,
                    draw_constellation: false,
                },
                timestamp: true,
                timestamp_pos: TimestampPos::BottomLeft as u32,
                night_mode: false,
                ircut_on: false,
                led_on: true,
                irled_on: false,
                flip: (false, false),
                fps: 25,
                brightness: 128,
                contrast: 128,
                sharpness: 128,
                saturation: 128,
                timezone: 9 * 3600,
                cap: false,
                logs: vec![],
            };
            if let Err(e) = save_to_file(app_state.clone()).await {
                log::error!("Failed to save the config : {}", e);
                panic!();
            }
            app_state
        }
    };
    let app_state_clone = app_state.clone();

    log::info!("Initializing system config");
    let atom_config = match load_atomconf().await {
        Ok(p) => {
            log::info!("Found system config file");
            p
        }
        Err(err) => {
            log::debug!("Failed to open the system config file {}", err);
            log::info!("Creating system config file");
            let atomconf = AtomConfig {
                netconf: NetworkConfig {
                    hostname: "atomskygaze".to_owned(),
                    ap_mode: false,
                    ssid: "".to_owned(),
                    psk: "".to_owned(),
                },
            };
            if let Err(e) = save_atomconf(atomconf.clone()).await {
                log::error!("Failed to save the systen config : {}", e);
                panic!();
            }
            atomconf
        }
    };

    let (tx, rx) = watch::channel(vec![]);
    let (logtx, logrx) = watch::channel(LogType::None);
    let app_state_common = Arc::new(Mutex::new(app_state));
    let atomconf_common = Arc::new(Mutex::new(atom_config));
    let (detected_tx, detected_rx) = mpsc::channel();
    let shutdown_flag = Arc::new(AtomicBool::new(false));

    if let Err(e) = gpio_init() {
        log::error!("Failed to initialize gpios : {}", e);
        panic!();
    }
    log::info!("gpio initialized");

    if app_state_clone.ircut_on {
        log::info!("turned on ircut leds");
        ircut_on().unwrap();
    }

    if app_state_clone.irled_on {
        log::info!("turned off ircut leds");
        irled_on().unwrap();
    }

    let app_state_common_instance = app_state_common.clone();
    let flag = shutdown_flag.clone();
    let wd_feeder = watchdog.make_instance();
    log::info!("Starting led_loop thread");
    let res = thread::Builder::new()
        .name("led_loop".to_string())
        .spawn(move || {
            let mut blue_on = false;
            loop {
                if flag.load(Ordering::Relaxed) {
                    log::info!("Stopping led_loop");
                    break;
                }

                let app_state = match app_state_common_instance.lock() {
                    Ok(guard) => guard,
                    Err(e) => {
                        warn!(
                            "app_state mutex lock error : {} at {}:{}",
                            e,
                            file!(),
                            line!()
                        );
                        continue;
                    }
                };

                if app_state.led_on {
                    if !app_state.detect {
                        drop(app_state);
                        if blue_on {
                            if let Err(e) = led_off(LEDType::Blue) {
                                log::error!("Failed to turn off blue led : {}", e);
                                panic!();
                            }
                            if let Err(e) = led_on(LEDType::Orange) {
                                log::error!("Failed to turn on orange led : {}", e);
                                panic!();
                            }
                        } else {
                            if let Err(e) = led_off(LEDType::Orange) {
                                log::error!("Failed to turn off orange led : {}", e);
                                panic!();
                            }
                            if let Err(e) = led_on(LEDType::Blue) {
                                log::error!("Failed to turn on blue led : {}", e);
                                panic!();
                            }
                        }

                        blue_on = !blue_on;
                    } else {
                        drop(app_state);
                        if let Err(e) = led_on(LEDType::Blue) {
                            log::error!("Failed to turn on blue led : {}", e);
                            panic!();
                        }
                        if let Err(e) = led_off(LEDType::Orange) {
                            log::error!("Failed to turn off orange led : {}", e);
                            panic!();
                        }
                    }
                } else {
                    drop(app_state);
                    if let Err(e) = led_off(LEDType::Blue) {
                        log::error!("Failed to turn off blue led : {}", e);
                        panic!();
                    }
                    if let Err(e) = led_off(LEDType::Orange) {
                        log::error!("Failed to turn off orange led : {}", e);
                        panic!();
                    }
                }
                if let Err(e) = wd_feeder.feed() {
                    log::error!("Failed to feed watchdog : {}", e);
                    panic!();
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        });

    if let Err(e) = res {
        log::error!("Failed to start led_loop : {}", e);
        panic!();
    }

    let app_state_common_instance1 = app_state_common.clone();
    let app_state_common_instance2 = app_state_common.clone();
    let app_state_common_instance3 = app_state_common.clone();
    let flag1 = shutdown_flag.clone();
    let flag2 = shutdown_flag.clone();
    let flag3 = shutdown_flag.clone();
    let flag4 = shutdown_flag.clone();
    unsafe {
        log::info!("Initializing IMP");
        if !imp_init(app_state_clone) {
            log::error!("imp_init failed");
            panic!();
        }
        log::info!("Logging all isp value");
        log_all_value();
        log::info!("Initializing framesource");
        if !imp_framesource_init() {
            log::error!("imp_framesource_init failed");
            panic!();
        }
        log::info!("Initializing encoders");
        if !imp_encoder_init() {
            log::error!("imp_encoder_init failed");
            panic!();
        }
        log::info!("Binding OSD");
        let (grp_num, font_handle) = imp_osd_bind();
        log::info!("Initializing jpeg");
        if !imp_jpeg_init() {
            log::error!("imp_jpeg_init failed");
            panic!();
        }
        log::info!("Initializing hevc");
        if !imp_hevc_init() {
            log::error!("imp_hevc_init failed");
            panic!();
        }
        log::info!("Initializing timelapse");
        if !imp_timelapse_init() {
            log::error!("imp_timelapse_init failed");
            panic!();
        }
        log::info!("Starting framesource");
        if !imp_framesource_start() {
            log::error!("imp_framesource_start failed");
            panic!();
        }
        log::info!("Initializing detection");
        if !init() {
            log::error!("detection init failed");
            panic!();
        }

        log::info!("Starting osd_loop thread");
        let res = thread::Builder::new()
            .name("osd_loop".to_string())
            .spawn(move || {
                imp_osd_start(grp_num, font_handle, app_state_common_instance1, flag1);
            });

        if let Err(e) = res {
            log::error!("Failed to start osd_loop : {}", e);
            panic!();
        }

        record_loops(detected_rx, app_state_common_instance2, flag2);

        log::info!("Starting jpeg_loop thread");
        let res = thread::Builder::new()
            .name("jpeg_loop".to_string())
            .spawn(|| {
                jpeg_start(tx, flag3);
            });

        if let Err(e) = res {
            log::error!("Failed to start jpeg_loop : {}", e);
            panic!();
        }

        log::info!("Starting detection_loop thread");
        let res = thread::Builder::new()
            .name("detection_loop".to_string())
            .spawn(move || {
                start(app_state_common_instance3, detected_tx, logtx, flag4);
            });

        if let Err(e) = res {
            log::error!("Failed to start detection_loop : {}", e);
            panic!();
        }
    };

    let assets_dir = PathBuf::from("/media/mmc/assets/web/");

    let flag = shutdown_flag.clone();
    let app = Router::new()
        .fallback_service(ServeDir::new(assets_dir).append_index_html_on_directories(true))
        .route("/ws", get(handler))
        .route("/download", get(download_file))
        .route("/view", get(view_file))
        .with_state((rx, app_state_common, atomconf_common, logrx, flag));

    // run it with hyper
    let listener = match tokio::net::TcpListener::bind("0.0.0.0:80").await {
        Ok(tcp) => tcp,
        Err(e) => {
            log::error!("Failed to bind TcpListener : {}", e);
            panic!();
        }
    };

    if let Err(e) = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    {
        log::error!("Axum serve failed : {}", e);
        panic!();
    }
}
