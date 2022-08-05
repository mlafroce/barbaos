use crate::devices::virtio::block_device::BlockDevice;
use crate::devices::virtio::DeviceError;
use crate::filesystem::SECTOR_SIZE;
use core::fmt::{Debug, Formatter};
use core::mem::{size_of, MaybeUninit};

pub struct LinuxPartition<'a> {
    device: &'a mut BlockDevice,
    first_sector: u64,
    superblock: Superblock,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct BlockGroup {
    bg_block_bitmap: u32, /* Bloquea el mapa de bits bloquea el mapa de bits rápido */
    bg_inode_bitmap: u32, /* Inodes bitmap block inode */
    bg_inode_table: u32,  /* Bloque de tabla de Inods */
    bg_free_blocks_count: u16, /* Conteo de bloques libres */
    bg_free_inodes_count: u16, /* Free inodes count */
    bg_used_dirs_count: u16, /* Directorios cuentan el número de directorios */
    bg_pad: u16,
    bg_reserved: [u32; 3],
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct Superblock {
    /// Inodes count
    s_inodes_count: i32,
    /// Blocks count
    s_blocks_count_lo: i32,
    /// Reserved blocks count
    s_r_blocks_count_lo: i32,
    /// Free blocks count
    s_free_blocks_count_lo: i32,
    /// Free inodes count
    s_free_inodes_count: i32,
    /// First Data Block
    s_first_data_block: i32,
    /// Block size
    s_log_block_size: i32,
    /// Allocation cluster size
    s_log_cluster_size: i32,
    /// Blocks per group
    s_blocks_per_group: i32,
    /// Clusters per group
    s_clusters_per_group: i32,
    /// Inodes per group
    s_inodes_per_group: i32,
    /// Mount time
    s_mtime: i32,
    /// Write time
    s_wtime: i32,
    /// Mount count
    s_mnt_count: i16,
    /// Maximal mount count
    s_max_mnt_count: i16,
    /// Magic signature
    s_magic: i16,
    /// File system state
    s_state: i16,
    /// Behaviour when detecting errors
    s_errors: i16,
    /// minor revision level
    s_minor_rev_level: i16,
    /// time of last check
    s_lastcheck: i32,
    /// max. time between checks
    s_checkinterval: i32,
    /// OS
    s_creator_os: i32,
    /// Revision level
    s_rev_level: i32,
    /// Default uid for reserved blocks
    s_def_resuid: i16,
    /// Default gid for reserved blocks
    s_def_resgid: i16,
    /// First non-reserved inode
    s_first_ino: i32,
    /// size of inode structure
    s_inode_size: i16,
    /// block group # of this superblock
    s_block_group_nr: i16,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct Inode {
    i_mode: u16,        /* File type and access rights */
    i_uid: u16,         /* Low 16 bits of Owner Uid */
    i_size: u32,        /* Size in bytes */
    i_atime: u32,       /* Access time */
    i_ctime: u32,       /* Creation time */
    i_mtime: u32,       /* Modification time */
    i_dtime: u32,       /* Deletion Time */
    i_gid: u16,         /* Low 16 bits of Group Id */
    i_links_count: u16, /* Links count */
    i_blocks: u32,      /* Blocks count */
    i_flags: u32,       /* File flags */
    i_osd1: u32,        /* OS Dependant flags */
    i_block: [u32; 15], /* Block pointers */
    i_generation: u32,  /* File version (NFS) */
    i_file_acl: u32,    /* Extended attributes */
    i_dir_acl: u32,     /* High 32 bits of 64bit file size */
    i_faddr: u32,       /* File fragment location */
    i_osd2: [u8; 12],   /* OS dependants bits */
}

impl<'a> LinuxPartition<'a> {
    pub fn new(device: &'a mut BlockDevice, first_sector: u64) -> Result<Self, DeviceError> {
        let offset = (first_sector + 2) * SECTOR_SIZE as u64;
        let superblock = LinuxPartition::read_superblock(device, offset)?;
        Ok(Self {
            device,
            first_sector,
            superblock,
        })
    }

    pub fn read_root(&mut self) -> Result<Inode, DeviceError> {
        self.read_inode(2)
    }

    pub fn read_inode(&mut self, inode_id: u64) -> Result<Inode, DeviceError> {
        let inode_id = inode_id - 1;
        let block_size = self.superblock.get_block_size() as u64;
        let block_id = inode_id / self.superblock.s_inodes_per_group as u64 + 2;
        let block = self.read_from_block::<BlockGroup>(block_id)?;

        let inode_table_offset = block.bg_inode_table as u64 * block_size;
        let inode_offset = inode_table_offset + inode_id * self.superblock.s_inode_size as u64;
        let inode = self.read_from_offset(inode_offset)?;
        Ok(inode)
    }

    pub fn read_from_block<T>(&mut self, block_id: u64) -> Result<T, DeviceError> {
        let block_id = block_id - 1;
        let block_start = core::cmp::max(2048, self.superblock.get_block_size() as u64 * block_id);
        self.read_from_offset(block_start)
    }

    fn read_from_offset<T>(&mut self, offset: u64) -> Result<T, DeviceError> {
        let partition_offset = self.first_sector * SECTOR_SIZE as u64;
        let offset = offset + partition_offset;
        let mut item = MaybeUninit::<T>::uninit();
        let data_size = size_of::<T>();
        let dest =
            unsafe { core::slice::from_raw_parts_mut(&mut item as *mut _ as *mut u8, data_size) };
        let mut buffer = [0u8; SECTOR_SIZE];
        self.device.read_sync(&mut buffer, offset)?;

        let buffer_offset = offset as usize % SECTOR_SIZE;
        dest.copy_from_slice(&buffer[buffer_offset..buffer_offset + size_of::<T>()]);
        let item = unsafe { item.assume_init() };
        Ok(item)
    }

    fn read_superblock(device: &mut BlockDevice, offset: u64) -> Result<Superblock, DeviceError> {
        let mut superblock_data = [0u8; SECTOR_SIZE];
        device.read_sync(&mut superblock_data, offset)?;
        let mut superblock = Superblock::default();
        unsafe {
            core::ptr::copy_nonoverlapping(
                superblock_data.as_ptr(),
                &mut superblock as *mut Superblock as *mut u8,
                size_of::<Superblock>(),
            );
        }
        Ok(superblock)
    }
}

impl Superblock {
    fn get_block_size(&self) -> u32 {
        1024 << self.s_log_block_size
    }
}

impl Debug for LinuxPartition<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Sector: {:?}, superblock: {:?}",
            self.first_sector, self.superblock
        )
    }
}
