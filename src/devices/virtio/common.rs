use crate::devices::virtio::block_device::BlockDevice;
use crate::{print, println};

const MMIO_VIRTIO_START: usize = 0x1000_1000;
const MMIO_VIRTIO_DEVICES: usize = 8;
const MMIO_VIRTIO_STRIDE: usize = 0x1000;
const MSG_LEN: usize = 13;

#[repr(usize)]
/// Layout from virtio-v1.1
pub enum VirtioMmioRegister {
    MagicValue = 0x00,
    Version = 0x04,
    DeviceId = 0x08,
    VendorId = 0x0c,
    HostFeatures = 0x10,
    //HostFeaturesSel = 0x14,
    GuestFeatures = 0x20,
    //GuestFeaturesSel = 0x24,
    GuestPageSize = 0x28,
    QueueSel = 0x030,
    QueueNumMax = 0x034,
    QueueNum = 0x038,
    //QueueAlign = 0x03c,
    QueuePFN = 0x040,
    QueueNotify = 0x050,
    Status = 0x70,
}

#[repr(u32)]
/// Status in virtio-v1.1
pub enum VirtioDeviceStatus {
    Reset = 0x00,
    Acknowledge = 0x01,
    Driver = 0x02,
    DriverOk = 0x04,
    FeaturesOk = 0x8,
}

#[derive(Debug)]
pub enum DeviceError {
    InvalidDevice,
    InitializationError,
    UnsupportedDevice(u32),
}

/// Readonly feature
pub const VIRTIO_BLK_F_RO: u32 = 0x20;
//const VIRTIO_F_RING_PACKED: u32 = 0x4;

pub const VIRTQ_DESC_F_NEXT: u16 = 1;
pub const VIRTQ_DESC_F_WRITE: u16 = 2;
//const VIRTQ_DESC_F_INDIRECT: u16 = 4;

pub const VIRTIO_BLK_T_IN: u32 = 0;
pub const VIRTIO_BLK_T_OUT: u32 = 1;

pub const DESCRIPTOR_HEADER_SIZE: u64 = 16;

#[derive(Copy, Clone)]
pub struct DeviceAddress {
    address: usize,
}

struct DeviceBuilder {
    address: DeviceAddress,
}

#[repr(C)]
#[derive(Debug)]
pub struct Descriptor {
    pub addr: u64,
    pub len: u32,
    pub flags: u16,
    pub next: u16,
}

#[repr(C)]
pub struct QueueAvailable<const RING_SIZE: usize> {
    pub flags: u16,
    pub idx: u16,
    pub ring: [u16; RING_SIZE],
    pub event: u16,
}

#[repr(C)]
pub struct UsedElem {
    pub id: u32,
    pub len: u32,
}

#[repr(C)]
pub struct QueueUsed<const RING_SIZE: usize> {
    pub flags: u16,
    pub idx: u16,
    pub ring: [UsedElem; RING_SIZE],
    pub event: u16,
}

impl DeviceAddress {
    pub fn new(address: usize) -> Self {
        Self { address }
    }

    pub fn read_register(&self, register: VirtioMmioRegister) -> u32 {
        let address = (self.address + register as usize) as *const u32;
        unsafe { address.read_volatile() }
    }

    pub fn write_register(&self, register: VirtioMmioRegister, value: u32) {
        let address = (self.address + register as usize) as *mut u32;
        unsafe { address.write_volatile(value) }
    }
}

impl DeviceBuilder {
    pub fn new(address: usize) -> Self {
        let address = DeviceAddress::new(address);
        Self { address }
    }

    pub fn valid(&self) -> bool {
        // 0x74726976 -> "triv" (virt little endian)
        self.address.read_register(VirtioMmioRegister::MagicValue) == 0x74726976
            && self.address.read_register(VirtioMmioRegister::Version) == 0x1
    }

    pub fn init_driver(self) -> Result<BlockDevice, DeviceError> {
        if !self.valid() {
            return Err(DeviceError::InvalidDevice);
        }
        self.address
            .write_register(VirtioMmioRegister::Status, VirtioDeviceStatus::Reset as u32);
        self.address.write_register(
            VirtioMmioRegister::Status,
            VirtioDeviceStatus::Acknowledge as u32,
        );
        let device_id = self.address.read_register(VirtioMmioRegister::DeviceId);
        match device_id {
            2 => BlockDevice::new(self.address),
            _ => Err(DeviceError::UnsupportedDevice(device_id)),
        }
    }
}

/// Sondeo de dispositivos en la memoria Virtio
pub fn probe() -> [Option<BlockDevice>; 8] {
    let mut devices = [None, None, None, None, None, None, None, None];
    #[allow(clippy::needless_range_loop)]
    for i in 0..MMIO_VIRTIO_DEVICES {
        let address = MMIO_VIRTIO_START + i * MMIO_VIRTIO_STRIDE;
        let builder = DeviceBuilder::new(address);
        if let Ok(device) = builder.init_driver() {
            println!("VirtIO device found at 0x{:x}", address);
            println!("Device type: {}", device.get_id());
            devices[i] = Some(device);
        }
    }
    devices
}
