* Init virtio

```
let mut devices : [MaybeUninit<DeviceDescription>; MMIO_VIRTIO_DEVICES] = unsafe { MaybeUninit::uninit().assume_init() };
for i in 0..MMIO_VIRTIO_DEVICES {
    let address = MMIO_VIRTIO_START + i * MMIO_VIRTIO_STRIDE;
    let description = DeviceDescription::new(address);
    // 0x74726976 -> "triv" (virt little endian)
    unsafe {
        if description.read_magic() == 0x74726976 {
        println!("VirtIO Device found: {:x}", address);
        println!("Device: {:x}", description.read_device_id());
        println!("Vendor: {:x}", description.read_vendor_id());
        }
    }
    devices[i] = MaybeUninit::new(description);
}
unsafe { core::mem::transmute::<_, [DeviceDescription; MMIO_VIRTIO_DEVICES]>(devices) };
```
