use crate::devices::shutdown;
use crate::devices::virtio::block_device::BlockDevice;
use crate::devices::virtio::DeviceError;
use crate::filesystem::linux::LinuxPartition;
use crate::filesystem::partition::{PartitionTable, PartitionType};
use crate::{print, println};

const MAX_PARTITIONS: u8 = 4;

pub fn load_disk(device: &mut BlockDevice) -> Result<(), DeviceError> {
    let mut buffer = [0u8; 512];
    device.read_sync(&mut buffer, 0)?;
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
        let mut partition = LinuxPartition::new(device, info.initial_sector as u64)?;
        println!("Ext2 metadata found: {:?}", partition);
        let root = partition.read_root()?;
        println!("Root inode: {:?}", root);
    }
    shutdown();
    Ok(())
}
