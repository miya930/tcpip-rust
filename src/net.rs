use log::{error, info, warn};

use crate::platform::linux::platform::{platform_init, platform_run, platform_shutdown};

pub fn net_init() -> bool {
    info!("initializing..");
    if !platform_init() {
        error!("platform_init() failure");
        return false;
    }
    info!("success");
    true
}

pub fn net_run() -> bool {
    info!("startup..");
    if !platform_run() {
        error!("platform_run() failure");
        return false;
    }
    info!("success");
    true
}

pub fn net_shutdown() -> bool {
    info!("shutting down..");
    if !platform_shutdown() {
        warn!("");
        return false;
    }
    info!("success");
    true
}
