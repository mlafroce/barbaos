use crate::devices::virtio::block_device::BlockDevice;
use crate::devices::virtio::DeviceError;
use crate::filesystem::linux::{Inode, LinuxPartition};
use crate::filesystem::partition::{PartitionTable, PartitionType};
use crate::filesystem::virtual_fs::{FileDescriptor, FilesystemDriver};
use crate::print;
use crate::utils::error::{IoError, IoResult};
use alloc::string::ToString;
use core::cell::RefCell;

const MAX_PARTITIONS: u8 = 4;

pub struct Ext2FilesystemDriver<'a> {
    device: &'a RefCell<BlockDevice>,
    partition_id: u8,
}

impl<'a> Ext2FilesystemDriver<'a> {
    pub fn new(device: &'a RefCell<BlockDevice>, partition_id: u8) -> Self {
        Self {
            device,
            partition_id,
        }
    }

    fn get_partition(&self) -> IoResult<LinuxPartition<'_>> {
        let mut buffer = [0u8; 512];
        self.device.borrow_mut().read_sync(&mut buffer, 0)?;
        let table = PartitionTable::new(buffer);
        // TODO: don't hardcode first partition
        let mut root_partition = None;
        for i in 0..MAX_PARTITIONS {
            let part_info = table.get_partition_info(i + 1);
            if part_info.partition_type == PartitionType::Linux {
                root_partition = Some(part_info);
                break;
            }
        }
        match root_partition {
            Some(info) => Ok(LinuxPartition::new(
                self.device,
                info.initial_sector as u64,
            )?),
            None => Err(IoError::FileNotExists),
        }
    }

    fn get_inode(&self, path: &str) -> IoResult<Inode> {
        let partition = self.get_partition().unwrap();
        let mut current_inode = partition.read_root()?;
        let path_iter = path.split("/").skip(1);
        for entry in path_iter {
            current_inode = partition
                .get_inode_block_iterator(current_inode)
                .and_then(|root_it| {
                    root_it
                        .get_entry_with_name(entry)
                        .ok_or(DeviceError::EntryNotFound)
                })
                .and_then(|file_entry| partition.get_inode_for_entry(file_entry))?;
        }
        Ok(current_inode)
    }

    fn print_file_contents(&self, inode: Inode) {
        let partition = self.get_partition().unwrap();
        let mut remaining_data = inode.i_size as u64;
        let file_block_iterator = partition.get_inode_block_iterator(inode).unwrap();
        let block_size = partition.get_block_size();
        for block in file_block_iterator {
            let block_data_size = core::cmp::min(remaining_data, block_size);
            for c in &block.data[0..block_data_size as usize] {
                print!("{}", *c as char);
            }
            remaining_data -= block_data_size;
        }
    }
}

impl FilesystemDriver for Ext2FilesystemDriver<'_> {
    fn open(&self, path: &str) -> IoResult<FileDescriptor> {
        let inode = self.get_inode(path);
        self.print_file_contents(inode.unwrap());
        let fd = FileDescriptor {
            path: path.to_string(),
            file_pos: 0,
            eof_flag: false,
        };
        Ok(fd)
    }
}
