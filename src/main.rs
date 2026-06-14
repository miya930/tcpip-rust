use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

use crate::net::Net;
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

    let mut net = Net::new();
    net.init();
    if !net.run() {
        error!("net.run() failure");
        return;
    }

    app_main(&terminate);
    net.shutdown();

    signals.handle.close();
    let _ = signals.join.join();
}

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
    let handle = signals.handle();

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

#[cfg(test)]
mod test {
    use log::{debug, error};
    use pretty_hex::PrettyHex;
    use std::{
        ops::Deref,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
    };

    use crate::{
        net::{Net, NetDevice, NetDeviceFactory},
        start_signal_thread,
    };

    const TEST_DATA: [u8; 48] = [
        0x45, 0x00, 0x00, 0x30, 0x00, 0x80, 0x00, 0x00, 0xff, 0x01, 0xbd, 0x4a, 0x7f, 0x00, 0x00,
        0x01, 0x7f, 0x00, 0x00, 0x01, 0x08, 0x00, 0x35, 0x64, 0x00, 0x80, 0x00, 0x01, 0x31, 0x32,
        0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30, 0x21, 0x40, 0x23, 0x24, 0x25, 0x5e, 0x26,
        0x2a, 0x28, 0x29,
    ];

    #[test]
    fn test_step1() {
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

        let mut net = Net::new();
        net.init();

        dummy_init(&mut net);

        if !net.run() {
            error!("net.run() failure");
            return;
        }

        app_main(&terminate, &net);
        net.shutdown();

        signals.handle.close();
        let _ = signals.join.join();
    }

    fn app_main(terminate: &Arc<AtomicBool>, net: &Net) {
        let dev = net.as_ref().devices.as_ref().expect("no device found");
        while !terminate.load(Ordering::Relaxed) {
            device_output(dev, 0x0800, TEST_DATA, TEST_DATA.len());
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        debug!("terminate");
    }

    pub fn dummy_init(net: &mut Net) {
        net.register(NetDeviceFactory::spawn_dummy_device());
    }

    fn device_output(dev: &NetDevice, d_type: u16, data: [u8; TEST_DATA.len()], len: usize) {
        debug!("dev={:?}, type={:04x}, len={}", dev.name, d_type, len);
        debug!("data: {:?}", data.hex_dump());
    }
}
