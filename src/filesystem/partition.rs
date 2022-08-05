use crate::filesystem::partition::PartitionType::Unsupported;
use crate::filesystem::SECTOR_SIZE;
use core::convert::TryInto;

#[derive(Debug, PartialEq)]
pub enum PartitionType {
    Free,
    Linux,
    Unsupported,
}

impl From<u8> for PartitionType {
    fn from(item: u8) -> Self {
        match item {
            0 => PartitionType::Free,
            0x83 => PartitionType::Linux,
            _ => Unsupported,
        }
    }
}

pub struct PartitionTable {
    data: [u8; SECTOR_SIZE],
}

#[derive(Debug)]
pub struct PartitionInfo {
    pub initial_sector: u32,
    pub size: u32,
    pub partition_type: PartitionType,
    pub booteable: bool,
}

impl PartitionTable {
    pub fn new(data: [u8; SECTOR_SIZE]) -> Self {
        Self { data }
    }

    pub fn is_mbr(&self) -> bool {
        self.data[0x1FE] == 0x55 && self.data[0x1FF] == 0xAA
    }

    pub fn get_partition_info(&self, partition: u8) -> PartitionInfo {
        assert!(partition > 0 && partition <= 4);
        let partition_address = 0x1AE + partition as usize * 0x10;
        let booteable = self.data[partition_address] == 0x80;
        let partition_type = PartitionType::from(self.data[partition_address + 4]);
        let initial_sector = u32::from_le_bytes(
            self.data[partition_address + 8..partition_address + 12]
                .try_into()
                .expect("slice error"),
        );
        let size = u32::from_le_bytes(
            self.data[partition_address + 12..partition_address + 16]
                .try_into()
                .expect("slice error"),
        );
        PartitionInfo {
            booteable,
            partition_type,
            initial_sector,
            size,
        }
    }
}
