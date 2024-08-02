use crate::devices::shutdown;
use crate::devices::virtio::DeviceError;
use crate::filesystem::virtual_fs::VirtualFsManager;

const MAX_PARTITIONS: u8 = 4;

pub fn load_disk() -> Result<(), DeviceError> {
    display_boot_file()?;
    shutdown();
    Ok(())
}

fn display_boot_file() -> Result<(), DeviceError> {
    VirtualFsManager::open("/boot/hello.md").unwrap();
    Ok(())
}
