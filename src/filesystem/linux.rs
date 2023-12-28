use crate::devices::virtio::block_device::BlockDevice;
use crate::devices::virtio::DeviceError;
use crate::filesystem::SECTOR_SIZE;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::fmt::{Debug, Formatter};
use core::mem::{size_of, MaybeUninit};

const MAX_BLOCK_SIZE: usize = 8_192;
const INODE_SINGLE_INDIRECT: usize = 12;
const INODE_DOUBLE_INDIRECT: usize = 13;
const INODE_TRIPLE_INDIRECT: usize = 14;

pub struct LinuxPartition<'a> {
    device: &'a RefCell<BlockDevice>,
    first_sector: u64,
    superblock: Superblock,
}

#[repr(u8)]
#[derive(Debug, Default)]
pub enum Ext2Filetype {
    /// Unknown File Type
    #[default]
    Unknown = 0,
    /// Regular File
    RegFile,
    /// Directory File
    Dir,
    /// Character Device
    Chrdev,
    /// Block Device
    Blkdev,
    /// Buffer File
    Fifo,
    /// Socket File
    Sock,
    /// Symbolic link,
    Symlink,
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
    pub i_size: u32,    /* Size in bytes */
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
    pub fn new(device: &'a RefCell<BlockDevice>, first_sector: u64) -> Result<Self, DeviceError> {
        let offset = (first_sector + 2) * SECTOR_SIZE as u64;
        let superblock = LinuxPartition::read_superblock(&mut device.borrow_mut(), offset)?;
        Ok(Self {
            device,
            first_sector,
            superblock,
        })
    }

    pub fn get_block_size(&self) -> u64 {
        self.superblock.get_block_size()
    }

    pub fn read_root(&self) -> Result<Inode, DeviceError> {
        self.read_inode(2)
    }

    pub fn get_inode_for_entry(&self, entry: DirectoryEntry) -> Result<Inode, DeviceError> {
        self.read_inode(entry.inode as u64)
    }

    fn read_inode(&self, inode_id: u64) -> Result<Inode, DeviceError> {
        let inode_id = inode_id - 1;
        let block_size = self.superblock.get_block_size();
        let block_id = inode_id / self.superblock.s_inodes_per_group as u64 + 2;
        let block = self.read_from_block::<BlockGroup>(block_id)?;

        let inode_table_offset = block.bg_inode_table as u64 * block_size;
        let inode_offset = inode_table_offset + inode_id * self.superblock.s_inode_size as u64;
        let inode = self.read_from_offset(inode_offset)?;
        Ok(inode)
    }

    fn read_from_block<T>(&self, block_id: u64) -> Result<T, DeviceError> {
        let block_id = block_id - 1;
        let block_start = core::cmp::max(2048, self.superblock.get_block_size() * block_id);
        self.read_from_offset(block_start)
    }

    fn read_from_offset<T>(&self, offset: u64) -> Result<T, DeviceError> {
        let partition_offset = self.first_sector * SECTOR_SIZE as u64;
        let offset = offset + partition_offset;
        let mut item = MaybeUninit::<T>::uninit();
        let data_size = size_of::<T>();
        let dest =
            unsafe { core::slice::from_raw_parts_mut(&mut item as *mut _ as *mut u8, data_size) };
        let mut buffer = [0u8; SECTOR_SIZE];
        self.device.borrow_mut().read_sync(&mut buffer, offset)?;

        let buffer_offset = offset as usize % SECTOR_SIZE;
        dest.copy_from_slice(&buffer[buffer_offset..buffer_offset + size_of::<T>()]);
        let item = unsafe { item.assume_init() };
        Ok(item)
    }

    pub fn read_datablock(&self, block_id: u64) -> Result<DataBlock, DeviceError> {
        let mut data = vec![0u8; self.superblock.get_block_size() as usize];
        let partition_offset = self.first_sector * SECTOR_SIZE as u64;
        let block_start = core::cmp::max(2048, self.superblock.get_block_size() * block_id);
        let offset = partition_offset + block_start;
        self.device.borrow_mut().read_sync(&mut data, offset)?;
        let block = DataBlock { data };
        Ok(block)
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

    pub fn get_inode_block_iterator(
        &'a self,
        inode: Inode,
    ) -> Result<InodeBlockIterator<'a>, DeviceError> {
        Ok(InodeBlockIterator {
            partition: self,
            inode,
            current_idx: 0,
        })
    }
}

impl Superblock {
    fn get_block_size(&self) -> u64 {
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

#[derive(Debug)]
pub struct DataBlock {
    pub data: Vec<u8>,
}

pub struct DirectoryIterator<'a> {
    block: &'a DataBlock,
    offset: usize,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct DirectoryEntry {
    inode: u32,
    rec_len: u16,
    name_len: u8,
    file_type: Ext2Filetype,
}
impl DataBlock {
    pub fn iter_directories(&self) -> DirectoryIterator {
        DirectoryIterator {
            block: self,
            offset: 0,
        }
    }

    pub unsafe fn read<T>(&self, offset: usize) -> Option<T> {
        let item_size = size_of::<T>();
        if offset + item_size > self.data.len() {
            return None;
        }
        let mut item = MaybeUninit::<T>::uninit();
        core::ptr::copy_nonoverlapping(
            self.data.as_ptr().add(offset),
            &mut item as *mut _ as *mut u8,
            item_size,
        );
        let item = item.assume_init();
        Some(item)
    }
}

pub struct InodeBlockIterator<'a> {
    partition: &'a LinuxPartition<'a>,
    inode: Inode,
    current_idx: usize,
}

impl InodeBlockIterator<'_> {
    pub fn get_entry_with_name(self, entry_name: &str) -> Option<DirectoryEntry> {
        for block in self {
            if let Some((entry, _)) = block
                .iter_directories()
                .find(|(_, name)| *name == entry_name)
            {
                return Some(entry);
            }
        }
        None
    }

    /// Returns the nth block_id.
    /// First `INODE_SINGLE_INDIRECT - 1`  entries in i_block[] are direct entries, which means
    /// that block `i_block[n]` file stores data.
    /// `i_block[INODE_SINGLE_INDIRECT]` points to a block with block ids (pointers) to file data
    /// `i_block[INODE_DOUBLE_INDIRECT]` points to a block with pointers to another block with pointers to data
    /// and finally `i_block[INODE_TRIPLE_INDIRECT]` points to a block with pointers to pointers to pointers
    ///
    /// ## Where is the nth block id?
    ///
    /// * Direct entries: this ones are easier, if 0 <= block_no <  INODE_SINGLE_INDIRECT -> block id = i_block[block_no]
    ///
    /// * Single indirect entries: if `block_no == INODE_SINGLE_INDIRECT`, it will be the first indirect entry.
    ///   lets say that `entries` is the number of pointers in a single block, we deduce that
    ///   `i_block[SINGLE_INDIRECT][0]` is the first indirect block (which is number `SINGLE_INDIRECT`) and
    ///   `i_block[SINGLE_INDIRECT][entries-1]` is the last single indirect entry (number `SINGLE_INDIRECT + entries - 1`).
    ///    So, if we lock for a block id in range [SINGLE_INDIRECT, SINGLE_INDIRECT + entries - 1], its a single indirect block
    ///
    /// * Double indirect entries: we continue with values above
    ///   `i_block[DOUBLE_INDIRECT][0][0]` is the first double indirect block (which is number `SINGLE_INDIRECT + entries`)
    ///   `i_block[DOUBLE_INDIRECT][1][0]` is the first pointer in the second pointer block (which is number `SINGLE_INDIRECT + entries + entries`)
    ///   `i_block[DOUBLE_INDIRECT][n][0]` is the first pointer in the nth pointer block (which is number `SINGLE_INDIRECT + entries * (n+1)`)
    ///   `i_block[DOUBLE_INDIRECT][entries - 1][0]` is the first pointer in the last pointer block (which is number `SINGLE_INDIRECT + entries * entries`)
    ///   `i_block[DOUBLE_INDIRECT][entries - 1][entries - 1]` is the last pointer in the last pointer block (which is number `SINGLE_INDIRECT + entries * entries + entries - 1`)
    ///   So, if we lock for a block id in range [SINGLE_INDIRECT + entries, SINGLE_INDIRECT + entries * entries + entries - 1], its a double indirect block
    ///
    /// * Triple indirect entries
    ///   `i_block[TRIPLE_INDIRECT][0][0][0]` is the first triple indirect block (which is number `SINGLE_INDIRECT + entries * (entries + 1)`)
    ///   `i_block[TRIPLE_INDIRECT][0][0][i]` is number `SINGLE_INDIRECT + entries * entries + entries + i`)
    ///   `i_block[TRIPLE_INDIRECT][0][j][i]` is number `SINGLE_INDIRECT + entries * entries + entries * (j + 1) + i`)
    ///   `i_block[TRIPLE_INDIRECT][k][j][i]` is number `SINGLE_INDIRECT + entries * entries * k + entries * entries + entries * (1 + j) + i`)
    ///   which equals to `SINGLE_INDIRECT + entries * entries * (k + 1) + entries * (j + 1) + i`
    fn get_block_id(&mut self, block_no: usize) -> Option<u32> {
        let entries = self.partition.superblock.get_block_size() as usize / size_of::<u32>(); // each entry is 4 bytes

        let first_double_indirect_entry = INODE_SINGLE_INDIRECT + entries;
        let first_triple_indirect_entry = first_double_indirect_entry + (entries * entries);

        let block_id = if block_no < INODE_SINGLE_INDIRECT {
            self.inode.i_block[self.current_idx]
        } else if block_no < first_double_indirect_entry {
            let indirect_block_id = self.inode.i_block[INODE_SINGLE_INDIRECT];
            let data_block = self
                .partition
                .read_datablock(indirect_block_id as u64)
                .ok()?;
            let offset = (self.current_idx - INODE_SINGLE_INDIRECT) * size_of::<u32>();
            unsafe { data_block.read::<u32>(offset)? }
        } else if block_no < first_triple_indirect_entry {
            let indirect_block_id = self.inode.i_block[INODE_DOUBLE_INDIRECT];
            let pointer_offset = (block_no - INODE_SINGLE_INDIRECT - entries) / entries;
            let pointer_block = self
                .partition
                .read_datablock(indirect_block_id as u64)
                .ok()?;
            let pointer_block_id =
                unsafe { pointer_block.read::<u32>(pointer_offset * size_of::<u32>())? };

            let pointer_offset = (block_no - INODE_SINGLE_INDIRECT) % entries;
            let pointer_block = self
                .partition
                .read_datablock(pointer_block_id as u64)
                .ok()?;
            unsafe { pointer_block.read::<u32>(pointer_offset * size_of::<u32>())? }
        } else {
            unimplemented!()
        };
        Some(block_id)
    }
}

impl Iterator for InodeBlockIterator<'_> {
    type Item = DataBlock;

    fn next(&mut self) -> Option<Self::Item> {
        let block_id = self.get_block_id(self.current_idx)?;
        self.current_idx += 1;
        if block_id != 0 {
            self.partition.read_datablock(block_id as u64).ok()
        } else {
            // TODO: support sparse files
            None
        }
    }
}

impl<'a> Iterator for DirectoryIterator<'a> {
    type Item = (DirectoryEntry, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(entry) = unsafe { self.block.read::<DirectoryEntry>(self.offset) } {
            if entry.rec_len == 0 {
                return None;
            }
            let name_offset = self.offset + size_of::<DirectoryEntry>();
            let name_bytes = &self.block.data[name_offset..name_offset + entry.name_len as usize];
            let name = core::str::from_utf8(name_bytes).unwrap_or("");

            self.offset += entry.rec_len as usize;
            Some((entry, name))
        } else {
            None
        }
    }
}
