use crate::imp_shutdown;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

pub async fn reboot(flag: Arc<Mutex<bool>>) {
    println!("Shutdown loops");
    match flag.lock() {
        Ok(mut guard) => {
            *guard = true;
        }
        Err(poisoned) => {
            let mut guard = poisoned.into_inner();
            *guard = true;
        }
    }

    sleep(Duration::from_secs(5)).await;

    println!("Shutdown IMP");
    unsafe {
        imp_shutdown();
    }

    println!("Shutdown System");
    let output = Command::new("reboot")
        .output()
        .expect("Failed to execute reboot command");

    if output.status.success() {
        println!("System reboot command executed successfully.");
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        println!("Failed to reboot system: {}", error_message);
    }
}
