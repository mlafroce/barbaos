use crate::devices::virtio::common::*;
use crate::mmu::riscv64::{PAGE_ORDER, PAGE_SIZE};
use crate::{print, println};
use alloc::boxed::Box;
use core::arch::asm;
use core::mem::{size_of, MaybeUninit};
use core::ptr::read_volatile;

/// Size of Virtio Queue ring
const VIRTIO_QUEUE_SIZE: usize = 1 << 7;

#[allow(dead_code)]
pub struct BlockDevice {
    address: super::common::DeviceAddress,
    queue: Box<SplitQueue<VIRTIO_QUEUE_SIZE>>,
    driver_idx: usize,
    device_idx: usize,
}

#[derive(Debug)]
#[repr(C)]
pub struct BlockRequest {
    req_type: u32,
    reserved: u32,
    sector: u64,
    data: *const u8,
    status: u8,
}

const fn split_queue_padding<const RING_SIZE: usize>() -> usize {
    (size_of::<Descriptor>() * RING_SIZE - size_of::<QueueAvailable<RING_SIZE>>()) % PAGE_SIZE
}

#[repr(C)]
struct SplitQueue<const RING_SIZE: usize>
where
    [(); split_queue_padding::<RING_SIZE>()]: Sized,
{
    descriptor_table: [Descriptor; RING_SIZE],
    available: QueueAvailable<RING_SIZE>,
    padding: [u8; split_queue_padding::<RING_SIZE>()],
    used: QueueUsed<RING_SIZE>,
}

pub fn test_disk(device: &mut BlockDevice, msg_len: usize) {
    let mut buffer = [0u8; 512];
    let request = device.new_request(buffer.as_mut_ptr(), 512, 0, false);
    while !request.is_finished() {
        unsafe { asm!("nop") }
    }
    println!("Reading from disk:");
    for c in buffer[0..msg_len].iter() {
        print!("{}", *c as char);
    }
    println!();
}

impl BlockDevice {
    pub fn new(address: DeviceAddress) -> Result<BlockDevice, DeviceError> {
        let mut status = VirtioDeviceStatus::Acknowledge as u32 | VirtioDeviceStatus::Driver as u32;
        address.write_register(VirtioMmioRegister::Status, status);
        // Negociación de features
        let host_features = address.read_register(VirtioMmioRegister::HostFeatures);

        // Si packed virtqueues está soportado, debería usarse, por el momento no está
        /*
        self.write_register(VirtioMmioRegister::HostFeaturesSel, 1);
        let host_features_ext = self.read_register(VirtioMmioRegister::HostFeatures);
        self.write_register(VirtioMmioRegister::HostFeaturesSel, 0);
        let packed_queue_support = host_features_ext & VIRTIO_F_RING_PACKED != 0;
         */
        let guest_features = host_features & !VIRTIO_BLK_F_RO;
        address.write_register(VirtioMmioRegister::GuestFeatures, guest_features);
        status |= VirtioDeviceStatus::FeaturesOk as u32;
        address.write_register(VirtioMmioRegister::Status, status);

        // Valido negociación
        let status_ok = address.read_register(VirtioMmioRegister::Status);
        if status_ok | VirtioDeviceStatus::FeaturesOk as u32 == 0 {
            return Err(DeviceError::InitializationError);
        }

        address.write_register(VirtioMmioRegister::QueueSel, 0);
        let queue_max_num = address.read_register(VirtioMmioRegister::QueueNumMax);
        println!("Max queue num: {}", queue_max_num);
        address.write_register(VirtioMmioRegister::QueueNum, VIRTIO_QUEUE_SIZE as u32);
        address.write_register(VirtioMmioRegister::GuestPageSize, PAGE_SIZE as u32);
        // TODO: queue de largo dinámico
        // Para mayor simplicidad usamos VirtQueue de tamaño fijo
        let queue = Box::new(SplitQueue::<VIRTIO_QUEUE_SIZE>::new());
        let queue_addr = &*queue as *const _ as u32;
        address.write_register(VirtioMmioRegister::QueuePFN, queue_addr >> PAGE_ORDER);

        status |= VirtioDeviceStatus::DriverOk as u32;
        address.write_register(VirtioMmioRegister::Status, status);
        let block_device = BlockDevice {
            address,
            queue,
            driver_idx: 0,
            device_idx: 0,
        };
        Ok(block_device)
    }

    pub fn get_id(&self) -> u32 {
        self.address.read_register(VirtioMmioRegister::DeviceId)
    }

    /// Creo un nuevo request en el heap ya que los descriptors necesitan la ubicación del mismo
    pub fn new_request(
        &mut self,
        buffer: *mut u8,
        size: usize,
        offset: usize,
        write: bool,
    ) -> Box<BlockRequest> {
        let sector = offset as u64 / 512;
        let request = Box::new(BlockRequest::new(buffer, sector, write));
        let header_desc = Descriptor {
            addr: &*request as *const _ as u64,
            len: DESCRIPTOR_HEADER_SIZE as u32,
            flags: 0,
            next: 0,
        };
        self.queue_descriptor(header_desc, true);
        let data_desc = Descriptor {
            addr: buffer as u64,
            len: size as u32,
            flags: VIRTQ_DESC_F_WRITE,
            next: 0,
        };
        self.queue_descriptor(data_desc, true);
        let status_desc = Descriptor {
            addr: &request.status as *const u8 as u64,
            len: 1,
            flags: VIRTQ_DESC_F_WRITE,
            next: 0,
        };
        self.queue_descriptor(status_desc, false);
        self.queue_available();
        self.address
            .write_register(VirtioMmioRegister::QueueNotify, 0);
        request
    }

    fn queue_descriptor(&mut self, mut desc: Descriptor, has_next: bool) {
        if has_next {
            desc.next = ((self.driver_idx + 1) % VIRTIO_QUEUE_SIZE) as u16;
            desc.flags |= VIRTQ_DESC_F_NEXT;
        }
        self.queue.descriptor_table[self.driver_idx] = desc;
        self.driver_idx = (self.driver_idx + 1) % VIRTIO_QUEUE_SIZE;
    }

    fn queue_available(&mut self) {
        let available_idx = self.queue.available.idx;
        // self.driver_idx apunta a status
        self.queue.available.ring[available_idx as usize] =
            (self.driver_idx as u16).wrapping_sub(3);
        self.queue.available.idx = available_idx.wrapping_add(1);
    }
}

impl BlockRequest {
    fn new(data: *mut u8, sector: u64, write: bool) -> Self {
        let req_type = if write {
            VIRTIO_BLK_T_OUT
        } else {
            VIRTIO_BLK_T_IN
        };
        BlockRequest {
            req_type,
            reserved: 0,
            sector,
            data,
            status: 0x7F,
        }
    }

    fn is_finished(&self) -> bool {
        let status;
        unsafe {
            status = read_volatile(&self.status);
        }
        status != 0x7F
    }
}

impl<const RING_SIZE: usize> SplitQueue<RING_SIZE>
where
    [(); split_queue_padding::<RING_SIZE>()]: Sized,
{
    fn new() -> Self {
        unsafe { MaybeUninit::zeroed().assume_init() }
    }
}
