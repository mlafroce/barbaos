use crate::mmu::map_table::EntryBits;
use crate::mmu::riscv64::{PageTable, PAGE_ORDER, PAGE_SIZE};
use crate::system::process::Process;
use crate::utils::NullTerminatedStr;
use alloc::vec::Vec;
use core::mem::size_of;
use core::ptr::copy_nonoverlapping;

const MAGIC_SIZE: usize = 4;
const ENTRY_ADDR_OFFSET: u64 = 24;
const SHT_PROGBITS: u32 = 0x1;
const SHT_STRTAB: u32 = 0x3;
const SHF_WRITE: u64 = 0x1;
const SHF_ALLOC: u64 = 0x2;
const SHF_EXECINSTR: u64 = 0x4;

pub enum ElfLoaderError {
    InvalidMagic,
}

#[repr(C)]
#[derive(Clone, Debug, Default)]
struct Elf64Header {
    ident: [u8; 16],
    filetype: u16,
    machine: u16,
    version: u32,
    entry: u32,
    phoff: u64,
    shoff: u64,
    flags: u32,
    ehsize: u16,
    phentsize: u16,
    phnum: u16,
    shentsize: u16,
    shnum: u16,
    shstrndx: u16,
}

#[repr(C)]
#[derive(Clone, Debug, Default)]
pub struct Elf64SectionHeader {
    name: u32,
    section_type: u32,
    flags: u64,
    addr: u64,
    offset: u64,
    size: u64,
    link: u32,
    info: u32,
    addralign: u64,
    entsize: u64,
}

pub struct SectionIterator {
    base_addr: *const u8,
    table_offset: u64,
    current_section: u16,
    n_sections: u16,
}

#[derive(Debug)]
pub struct ElfLoader {
    base_addr: *const u8,
    header: Elf64Header,
}

impl ElfLoader {
    pub fn new(base_addr: *const u8) -> Result<Self, ElfLoaderError> {
        let src_header = unsafe { &*(base_addr as *const Elf64Header) };
        if src_header.ident[0..MAGIC_SIZE] == [0x7f, b'E', b'L', b'F'] {
            let header = src_header.clone();
            Ok(Self { base_addr, header })
        } else {
            Err(ElfLoaderError::InvalidMagic)
        }
    }

    pub fn get_section_iterator(&self) -> SectionIterator {
        SectionIterator::new(self.base_addr)
    }

    pub fn into_process(self, parent_page_table: &PageTable) -> Option<Process<'_>> {
        let mut process = Process::create(parent_page_table);
        let mut memory_sections = self
            .get_section_iterator()
            .filter(|sec| (sec.flags & SHF_ALLOC) != 0)
            .collect::<Vec<_>>();
        if memory_sections.is_empty() {
            panic!("Empty ELF");
        }
        memory_sections.sort_by(|l, r| l.addr.cmp(&r.addr));
        let pages = self.needed_pages(&memory_sections);
        let pages = parent_page_table.zalloc(pages)?;
        let mut current_page: isize = -1;
        let mut last_end_page = memory_sections[0].addr;
        for section in memory_sections {
            let section_start_page = section.addr >> PAGE_ORDER;
            let section_end_page = (section.addr + section.size) >> PAGE_ORDER;
            let clean_page = section_start_page != last_end_page;
            let section_pages = section_end_page - section_start_page + 1;
            if clean_page {
                current_page += 1;
            }
            let dest_offset =
                (current_page << PAGE_ORDER) as usize + (section.addr as usize % PAGE_SIZE);
            if section.section_type == SHT_PROGBITS {
                unsafe {
                    let src = self.base_addr.add(section.offset as usize);
                    let dst = pages.as_ptr().add(dest_offset);
                    copy_nonoverlapping(src, dst, section.size as usize);
                }
            }
            last_end_page = section_end_page;
            let map_start = if clean_page { 0 } else { 1 };
            for i in map_start..section_pages {
                let virt_page_offset = PAGE_SIZE * i as usize;
                let vaddr = section.addr as usize + virt_page_offset;
                let phys_page_offset = PAGE_SIZE * (current_page as usize + i as usize);
                let paddr = unsafe { pages.as_ptr().add(phys_page_offset) };
                let bits = if (section.flags & SHF_EXECINSTR) != 0 {
                    EntryBits::UserReadExecute
                } else if (section.flags & SHF_WRITE) != 0 {
                    EntryBits::UserReadWrite
                } else {
                    EntryBits::UserReadExecute
                };
                process.map_memory(vaddr, paddr as usize, bits.val(), 0);
            }
            current_page += section_pages as isize - 1;
        }
        process.program_counter = self.header.entry as usize;
        Some(process)
    }

    fn needed_pages(&self, sections: &Vec<Elf64SectionHeader>) -> usize {
        let mut counter = 0;
        let mut last_end_page = u64::MAX;
        for section in sections {
            let section_start_page = section.addr >> PAGE_ORDER;
            let section_end_page = (section.addr + section.size) >> PAGE_ORDER;
            let clean = section_start_page == last_end_page;
            if !clean {
                counter += 1;
            }
            counter += section_end_page - section_start_page;
            last_end_page = section_end_page;
        }
        counter as usize
    }
}

impl Elf64SectionHeader {
    pub fn is_section_name(&self, loader: &ElfLoader, name: &str) -> bool {
        let str_table_section_offset = loader.base_addr as u64
            + loader.header.shoff
            + loader.header.shstrndx as u64 * size_of::<Elf64SectionHeader>() as u64;
        let str_table_section =
            unsafe { &*(str_table_section_offset as *const Elf64SectionHeader) };
        let name_offset = loader.base_addr as u64 + str_table_section.offset + self.name as u64;
        let name_str = unsafe { NullTerminatedStr::as_str(name_offset as *const u8) };
        name_str.eq(name)
    }
}

impl SectionIterator {
    fn new(base_addr: *const u8) -> Self {
        let header = unsafe { &*(base_addr as *const Elf64Header) };
        let table_offset = header.shoff;
        let n_sections = header.shnum;
        let current_section = 0;
        Self {
            base_addr,
            table_offset,
            n_sections,
            current_section,
        }
    }
}

impl Iterator for SectionIterator {
    type Item = Elf64SectionHeader;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_section < self.n_sections {
            let section_offset = self.base_addr as u64
                + self.table_offset
                + self.current_section as u64 * size_of::<Elf64SectionHeader>() as u64;
            let src_section = unsafe { &*(section_offset as *const Elf64SectionHeader) };
            self.current_section += 1;
            Some(src_section.clone())
        } else {
            None
        }
    }
}
