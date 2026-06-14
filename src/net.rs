use log::{debug, error, info, warn};

use crate::platform::linux::platform::{platform_init, platform_run, platform_shutdown};

const IFNAMSIZE: usize = 10;
const NET_DEVICE_ADDR_LEN: usize = 16;

const NET_DEVICE_TYPE_DUMMY: u16 = 0x0000;
const NET_DEVICE_TYPE_LOOPBACK: u16 = 0x0001;
const NET_DEVICE_TYPE_ETHERNET: u16 = 0x0002;

const NET_DEVICE_FLAG_UP: u16 = 0x0001;
const NET_DEVICE_FLAG_LOOPBACK: u16 = 0x0010;
const NET_DEVICE_FLAG_BROADCAST: u16 = 0x0020;
const NET_DEVICE_FLAG_P2P: u16 = 0x0040;
const NET_DEVICE_FLAG_NEED_ARP: u16 = 0x0100;

pub fn device_output(dev: &NetDevice, len: u16) {
    debug!(
        "dev={:?}, type=0x{:04x}, len={:?}",
        dev.name, dev.d_type, len
    );

    if dev.is_up() {
        error!("not opened, dev={:?}", dev.name);
        return;
    }
    if dev.mtu < len {
        error!(
            "too long, dev={:?}, mtu={:?}, len={:?}",
            dev.name, dev.mtu, len
        );
    }
}

pub struct Net {
    devices: NetDevices,
}
impl Net {
    pub fn new() -> Self {
        Self {
            devices: NetDevices {
                devices: None,
                index: 0x00,
            },
        }
    }

    pub fn init(&self) -> bool {
        info!("initializing..");
        if !platform_init() {
            error!("platform_init() failure");
            return false;
        }
        info!("success");
        true
    }

    pub fn register(&mut self, dev: Box<NetDevice>) {
        self.devices.register(dev);
    }

    pub fn run(&mut self) -> bool {
        info!("startup..");
        if !platform_run() {
            error!("platform_run() failure");
            return false;
        }

        self.devices.iter_mut().for_each(|dev| dev.open());

        info!("success");
        true
    }

    pub fn shutdown(&mut self) -> bool {
        info!("shutting down..");
        if !platform_shutdown() {
            warn!("");
            return false;
        }

        self.devices.iter_mut().for_each(|dev| dev.close());

        info!("success");
        true
    }

    pub fn as_ref(&self) -> &NetDevices {
        &self.devices
    }
}

#[derive(Clone, Debug)]
pub struct NetDevice {
    pub next: Option<Box<NetDevice>>,
    pub index: usize,
    pub name: [u8; IFNAMSIZE],
    pub d_type: u16,
    pub mtu: u16,
    pub flags: u16,
    pub hlen: u16,
    pub alen: u16,
    pub addr: [u8; NET_DEVICE_ADDR_LEN],
    pub broadcast: [u8; NET_DEVICE_ADDR_LEN],
}
impl NetDevice {
    pub fn new() -> Self {
        NetDevice {
            next: None,
            index: 0,
            name: [0x00; IFNAMSIZE],
            d_type: NET_DEVICE_TYPE_DUMMY,
            mtu: 1000,
            flags: 0x0000,
            hlen: 0x00,
            alen: 0x00,
            addr: [0x00; NET_DEVICE_ADDR_LEN],
            broadcast: [0x00; NET_DEVICE_ADDR_LEN],
        }
    }

    pub fn is_up(&self) -> bool {
        (self.flags & NET_DEVICE_FLAG_UP) != 0x00
    }

    pub fn open(&mut self) {
        info!("open dev={:?}", self.name);
        if self.is_up() {
            error!("already opened, dev={:?}", self.name);
            return;
        }
        self.flags |= NET_DEVICE_FLAG_UP;
    }

    pub fn close(&mut self) {
        info!("close dev={:?}", self.name);
        if !self.is_up() {
            error!("not opened, dev={:?}", self.name);
            return;
        }
        self.flags &= !NET_DEVICE_FLAG_UP;
    }
}

#[derive(Clone, Debug)]
pub struct NetDevices {
    pub devices: Option<Box<NetDevice>>,
    index: usize,
}
impl NetDevices {
    pub fn new() -> Self {
        NetDevices {
            devices: None,
            index: 0x00,
        }
    }

    pub fn register(&mut self, mut dev: Box<NetDevice>) {
        dev.index = self.index;
        self.index += 1;

        info!(
            "name: {:?}, sizeof({:?}), net{:?}",
            dev.name, dev.name, dev.index
        );

        dev.next = self.devices.take();
        self.devices = Some(dev);
    }

    pub fn iter(&self) -> Iter<'_> {
        Iter {
            cur: self.devices.as_deref(),
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<'_> {
        IterMut {
            cur: self.devices.as_deref_mut(),
        }
    }
}

pub struct Iter<'a> {
    cur: Option<&'a NetDevice>,
}
impl<'a> Iterator for Iter<'a> {
    type Item = &'a NetDevice;

    fn next(&mut self) -> Option<Self::Item> {
        let dev = self.cur?;
        self.cur = dev.next.as_deref();
        Some(dev)
    }
}

pub struct IterMut<'a> {
    cur: Option<&'a mut NetDevice>,
}
impl<'a> Iterator for IterMut<'a> {
    type Item = &'a mut NetDevice;

    fn next(&mut self) -> Option<Self::Item> {
        let dev = self.cur.take()?;
        let p: *mut NetDevice = dev;
        self.cur = unsafe { (*p).next.as_deref_mut() };
        Some(unsafe { &mut *p })
    }
}

pub struct NetDeviceFactory {}
impl NetDeviceFactory {
    pub fn spawn_dummy_device() -> Box<NetDevice> {
        let mut dev = Box::new(NetDevice::new());

        dev.d_type = NET_DEVICE_TYPE_DUMMY;
        dev.mtu = 128;
        dev.hlen = 0; /* no header */
        dev.alen = 0; /* no address */

        dev
    }
}
