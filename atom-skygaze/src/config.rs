use crate::AppState;
use crate::DetectionConfig;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub mask: Vec<u8>, // len: 18*32
    pub detect: bool,
    pub detection_config: DetectionConfig,
    pub timestamp: bool,
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
}

pub async fn save_to_file(app_state: AppState) -> std::io::Result<()> {
    let app_config = AppConfig {
        mask: app_state.mask,
        detect: app_state.detect,
        detection_config: app_state.detection_config,
        timestamp: app_state.timestamp,
        night_mode: app_state.night_mode,
        ircut_on: app_state.ircut_on,
        led_on: app_state.led_on,
        irled_on: app_state.irled_on,
        flip: app_state.flip,
        fps: app_state.fps,
        brightness: app_state.brightness,
        contrast: app_state.contrast,
        sharpness: app_state.sharpness,
        saturation: app_state.saturation,
    };

    let mut file = File::create("/media/mmc/config.toml").await?;
    let toml = toml::to_string(&app_config).unwrap();
    file.write_all(toml.as_bytes()).await?;
    file.sync_all().await
}

pub async fn load_from_file() -> std::io::Result<AppState> {
    let content = tokio::fs::read_to_string("/media/mmc/config.toml").await?;
    let app_config: AppConfig = toml::from_str(&content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    Ok(AppState {
        mask: app_config.mask,
        detect: app_config.detect,
        detection_config: app_config.detection_config,
        timestamp: app_config.timestamp,
        night_mode: app_config.night_mode,
        ircut_on: app_config.ircut_on,
        led_on: app_config.led_on,
        irled_on: app_config.irled_on,
        flip: app_config.flip,
        fps: app_config.fps,
        brightness: app_config.brightness,
        contrast: app_config.contrast,
        sharpness: app_config.sharpness,
        saturation: app_config.saturation,
        logs: vec![],
    })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AtomConfig {
    pub netconf: NetworkConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetworkConfig {
    pub hostname: String,
    pub ap_mode: bool,
    pub ssid: String,
    pub psk: String,
}

pub async fn load_atomconf() -> std::io::Result<AtomConfig> {
    let content = tokio::fs::read_to_string("/media/mmc/atom_config.toml").await?;
    let atomconf: AtomConfig = toml::from_str(&content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    Ok(atomconf)
}

pub async fn save_atomconf(atomconf: AtomConfig) -> std::io::Result<()> {
    let mut file = File::create("/media/mmc/atom_config.toml").await?;
    let toml = toml::to_string(&atomconf).unwrap();
    file.write_all(toml.as_bytes()).await?;
    file.sync_all().await
}
