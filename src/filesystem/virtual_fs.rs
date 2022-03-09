use crate::devices::virtio::common::DeviceManager;
use crate::devices::DeviceId;
use crate::filesystem::ext2_fs_driver::Ext2FilesystemDriver;
use crate::utils::error::IoResult;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::UnsafeCell;

pub trait FilesystemDriver {
    fn open(&self, path: &str) -> IoResult<FileDescriptor>;
}

#[derive(Debug, Default)]
pub enum FilesystemType {
    Memory,
    Ext3 {
        device_id: DeviceId,
        partition_id: u8,
    },
    #[default]
    Unknown,
}

#[derive(Debug, Default)]
pub struct MountPoint {
    pub path: String,
    pub fs_type: FilesystemType,
}

#[derive(Debug, Default)]
struct VirtualFilesystem {
    mount_points: Vec<MountPoint>,
    null_mountpoint: MountPoint,
}

#[derive(Debug)]
pub struct FileDescriptor {
    pub path: String,
    pub file_pos: usize,
    pub eof_flag: bool,
}

static VIRTUAL_FILESYSTEM: VirtualFsManager = VirtualFsManager::empty();

pub struct VirtualFsManager {
    virtual_fs: UnsafeCell<Option<VirtualFilesystem>>,
}

impl VirtualFsManager {
    const fn empty() -> Self {
        let virtual_fs = UnsafeCell::new(None);
        Self { virtual_fs }
    }

    pub fn init() {
        let virtual_fs = Some(VirtualFilesystem::default());
        let vfs_ptr = VIRTUAL_FILESYSTEM.virtual_fs.get();
        unsafe { *vfs_ptr = virtual_fs };
    }

    pub fn push_mount_point(mount_point: MountPoint) {
        let vfs_ptr = VIRTUAL_FILESYSTEM.virtual_fs.get();
        let virtfs = unsafe { (*vfs_ptr).as_mut().unwrap() };
        virtfs.mount_points.push(mount_point);
        virtfs
            .mount_points
            .sort_by(|lhs, rhs| lhs.path.cmp(&rhs.path));
    }

    pub fn open(path: &str) -> IoResult<FileDescriptor> {
        let vfs_ptr = VIRTUAL_FILESYSTEM.virtual_fs.get();
        let virtfs = unsafe { (*vfs_ptr).as_mut().unwrap() };
        let mount_point = virtfs.get_mount_point(path);
        let driver = virtfs.get_driver(&mount_point.fs_type);
        driver.open(path)
    }
}

unsafe impl Sync for VirtualFsManager {}

impl VirtualFilesystem {
    fn get_mount_point(&self, path: &str) -> &MountPoint {
        let res = self
            .mount_points
            .iter()
            .rfind(|&mp| path.starts_with(&mp.path));
        res.unwrap_or(&self.null_mountpoint)
    }

    fn get_driver(&self, fs_type: &FilesystemType) -> impl FilesystemDriver {
        match fs_type {
            FilesystemType::Ext3 {
                device_id,
                partition_id,
            } => {
                if let Some(device) = DeviceManager::get_device(*device_id) {
                    Ext2FilesystemDriver::new(device, *partition_id)
                } else {
                    unimplemented!("Device not found")
                }
            }
            _ => unimplemented!(),
        }
    }
}
