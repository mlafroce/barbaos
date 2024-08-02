use crate::devices::shutdown;
use crate::devices::virtio::DeviceError;
use crate::filesystem::virtual_fs::FilesystemType::Ext3;
use crate::filesystem::virtual_fs::{MountPoint, VirtualFsManager};
use alloc::string::ToString;

const MAX_PARTITIONS: u8 = 4;

pub fn load_disk() -> Result<(), DeviceError> {
    VirtualFsManager::init();
    let mount_point = MountPoint {
        path: "/".to_string(),
        fs_type: Ext3 {
            device_id: 0,
            partition_id: 0,
        },
    };
    VirtualFsManager::push_mount_point(mount_point);
    display_boot_file()?;
    shutdown();
    Ok(())
}

fn display_boot_file() -> Result<(), DeviceError> {
    VirtualFsManager::open("/boot/hello.md").unwrap();
    Ok(())
}
