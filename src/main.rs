use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

use crate::net::{net_init, net_run, net_shutdown};
use log::{debug, error, info};
use signal_hook::consts::{SIGHUP, SIGINT};
use signal_hook::iterator::{Handle, Signals};

mod net;
mod platform;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let terminate = Arc::new(AtomicBool::new(false));
    let signals = match start_signal_thread(Arc::clone(&terminate)) {
        Ok(s) => s,
        Err(e) => {
            error!("signal setup failure: {e}");
            return;
        }
    };

    if !setup() {
        error!("setup() failure");
        return;
    }

    app_main(&terminate);

    if !cleanup() {
        error!("cleanup failure");
    }

    signals.handle.close();
    let _ = signals.join.join();
}

/// アプリ本体。終了要求が来るまで回す。
fn app_main(terminate: &Arc<AtomicBool>) {
    while !terminate.load(Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    debug!("terminate");
}

struct SignalThread {
    handle: Handle,
    join: JoinHandle<()>,
}

fn start_signal_thread(terminate: Arc<AtomicBool>) -> std::io::Result<SignalThread> {
    let mut signals = Signals::new([SIGHUP, SIGINT])?;
    let handle = signals.handle(); // forever() を止めるため先に取得

    let join = std::thread::spawn(move || {
        for sig in signals.forever() {
            match sig {
                SIGHUP => {
                    info!("SIGHUP received");
                    terminate.store(true, Ordering::SeqCst);
                }
                SIGINT => {
                    debug!("SIGINT received");
                    terminate.store(true, Ordering::SeqCst);
                }
                _ => unreachable!(),
            }
        }
        info!("signal thread stopped");
    });

    Ok(SignalThread { handle, join })
}

fn setup() -> bool {
    if !net_init() {
        error!("net_init() failure");
        return false;
    }
    if !net_run() {
        error!("net_run() failure");
        return false;
    }
    true
}

fn cleanup() -> bool {
    info!("cleanup protocol stack..");
    if !net_shutdown() {
        error!("net_shutdown() failure");
        return false;
    }
    true
}
