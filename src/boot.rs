use crate::devices::shutdown;
use crate::devices::virtio::block_device::BlockDevice;
use crate::devices::virtio::DeviceError;
use crate::filesystem::linux::LinuxPartition;
use crate::filesystem::partition::{PartitionTable, PartitionType};
use crate::{print, println};
use core::cell::RefCell;

const MAX_PARTITIONS: u8 = 4;

pub fn load_disk(device: &RefCell<BlockDevice>) -> Result<(), DeviceError> {
    let mut buffer = [0u8; 512];
    device.borrow_mut().read_sync(&mut buffer, 0)?;
    let table = PartitionTable::new(buffer);
    if table.is_mbr() {
        println!("Found MBR");
    }
    // TODO: don't hardcode first partition
    let mut root_partition = None;
    println!("Iterating partitions...");
    for i in 0..MAX_PARTITIONS {
        let part_info = table.get_partition_info(i + 1);
        if part_info.partition_type == PartitionType::Linux {
            println!("Partition {} is a Linux (EXT) partition", i + 1);
            root_partition = Some(part_info);
            break;
        }
    }
    if let Some(info) = root_partition {
        let partition = LinuxPartition::new(device, info.initial_sector as u64)?;
        display_root(&partition)?;
        display_boot_file(&partition)?;
    }
    shutdown();
    Ok(())
}

fn display_root<'a>(partition: &'a LinuxPartition<'a>) -> Result<(), DeviceError> {
    let root = partition.read_root()?;
    let block_iterator = partition.get_inode_block_iterator(root)?;
    println!("Iterating root directory...");
    for block in block_iterator {
        for entry in block.iter_directories() {
            println!("Entry: {:?}", entry);
        }
    }
    Ok(())
}

fn display_boot_file<'a>(partition: &'a LinuxPartition<'a>) -> Result<(), DeviceError> {
    let root = partition.read_root()?;
    let file_inode = partition
        .get_inode_block_iterator(root)
        .ok()
        .and_then(|root_it| root_it.get_entry_with_name("boot"))
        .and_then(|boot_entry| partition.get_inode_for_entry(boot_entry).ok())
        .and_then(|inode| partition.get_inode_block_iterator(inode).ok())
        .and_then(|boot_it| boot_it.get_entry_with_name("hello.md"))
        .and_then(|file_entry| partition.get_inode_for_entry(file_entry).ok());
    if let Some(inode) = file_inode {
        let mut remaining_data = inode.i_size as u64;
        let file_block_iterator = partition.get_inode_block_iterator(inode)?;
        println!("Boot file size: {} bytes", remaining_data);
        let block_size = partition.get_block_size();
        for block in file_block_iterator {
            let block_data_size = core::cmp::min(remaining_data, block_size);
            for c in &block.data[0..block_data_size as usize] {
                print!("{}", *c as char);
            }
            remaining_data -= block_data_size;
        }
    }
    Ok(())
}
