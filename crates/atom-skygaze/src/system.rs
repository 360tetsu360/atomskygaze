use crate::imp_shutdown;
use log::{error, info};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{sleep, Duration};

pub async fn reboot(flag: Arc<AtomicBool>) {
    info!("Shutdown loops");
    flag.store(true, Ordering::Relaxed);

    sleep(Duration::from_secs(5)).await;

    info!("Shutdown IMP");
    unsafe {
        if !imp_shutdown() {
            error!("Failed to shutdown imp");
            panic!();
        }
    }

    info!("Shutdown System");
    let output = match Command::new("reboot").output() {
        Ok(o) => o,
        Err(e) => {
            error!("Failed to execute reboot command : {}", e);
            panic!();
        }
    };

    if output.status.success() {
        info!("System reboot command executed successfully.");
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        error!("Failed to reboot system: {}", error_message);
        panic!();
    }
}
