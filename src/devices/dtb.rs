use crate::utils::error::OsError;
use crate::utils::NullTerminatedStr;
use crate::{print, println};
use core::mem::size_of;
use core::ptr::copy_nonoverlapping;

const FDT_BEGIN_NODE: u8 = 1;
const FDT_END_NODE: u8 = 2;
const FDT_PROP: u8 = 3;
const FDT_NOP: u8 = 4;
const FDT_END: u8 = 9;

const DTB_MAGIC: u32 = 0xd00dfeed;

#[derive(Debug)]
#[repr(C)]
struct FdtHeader {
    magic: u32,
    totalsize: u32,
    off_dt_struct: u32,
    off_dt_strings: u32,
    off_mem_rsvmap: u32,
    version: u32,
    last_comp_version: u32,
    boot_cpuid_phys: u32,
    size_dt_strings: u32,
    size_dt_struct: u32,
}

#[repr(C)]
struct FdtReserveEntry {
    address: u64,
    size: u64,
}

pub struct DtbReader {
    address: usize,
    header: FdtHeader,
}

impl DtbReader {
    pub fn new(address: *const u8) -> Result<DtbReader, OsError> {
        let mut header = core::mem::MaybeUninit::<FdtHeader>::uninit();
        let header = unsafe {
            copy_nonoverlapping(
                address,
                &mut header as *mut _ as *mut u8,
                size_of::<FdtHeader>(),
            );
            header.assume_init()
        };
        if header.magic.to_be() != DTB_MAGIC {
            return Err(OsError::DtbError);
        }
        let address = address as usize;
        Ok(Self { address, header })
    }

    pub fn print_boot_info(&self) {
        self.print_memory_reservations();
        self.print_nodes();
    }

    fn print_memory_reservations(&self) {
        let mut offset = self.address + self.header.off_mem_rsvmap.to_be() as usize;
        println!("Reading reserved memory blocks...");
        let mem_reserve_iter = core::iter::from_fn(move || {
            let mut mem_res = core::mem::MaybeUninit::<FdtReserveEntry>::uninit();
            let mem_res = unsafe {
                core::ptr::copy_nonoverlapping(
                    offset as *const u8,
                    &mut mem_res as *mut _ as *mut u8,
                    size_of::<FdtReserveEntry>(),
                );
                mem_res.assume_init()
            };
            offset += size_of::<FdtReserveEntry>();
            if mem_res.address != 0 && mem_res.size != 0 {
                Some(mem_res)
            } else {
                None
            }
        });
        mem_reserve_iter.for_each(|mem_res| {
            println!(
                "Reserved memory block at [{:x}..{:x}]",
                mem_res.address,
                mem_res.address + mem_res.size
            );
        });
    }

    #[cfg(target_arch = "arm")]
    fn print_nodes(&self) {
        println!("DTB info...");
        self.print_prop("Compatible", "compatible");
        self.print_prop("Model     ", "model");
        let node_iterator = NodeIterator::new(self);
        if let Some(FdtNode::PropNode(_, data, _)) = node_iterator
            .skip_while(|node| !node.starts_with("cpu@"))
            .find(|node| node.has_prop_or_label("compatible"))
        {
            let data_str = unsafe { NullTerminatedStr::as_str(data) };
            println!("CPU Compatible: {}", data_str)
        }
        let info = self.get_memory_info();
        println!("Memory start: 0x{:x}", info[0].to_be());
        println!("Memory size : 0x{:x}", info[1].to_be());
    }

    #[cfg(target_arch = "riscv64")]
    fn print_nodes(&self) {
        println!("DTB info...");
        self.print_prop("Compatible", "compatible");
        self.print_prop("Model     ", "model");
        self.print_prop("ISA       ", "riscv,isa");
        self.print_prop("MMU type  ", "mmu-type");
        let info = self.get_memory_info();
        println!("Memory start: 0x{:x}", info[0].to_be());
        println!("Memory size : 0x{:x}", info[1].to_be());
    }

    fn print_prop(&self, display: &str, prop_label: &str) {
        let mut node_iterator = NodeIterator::new(self);
        if let Some(FdtNode::PropNode(_, prop, _)) =
            node_iterator.find(|node| node.has_prop_or_label(prop_label))
        {
            let prop = unsafe { NullTerminatedStr::as_str(prop) };
            println!("{}: {}", display, prop)
        }
    }

    // Data[1] tiene el inicio de la memoria, data[3] el tamaño (O serán 2 enteros de 64 bits?)
    pub fn get_memory_info(&self) -> [usize; 2] {
        let mut res = [0; 2];
        let node_iterator = NodeIterator::new(self);
        if let Some(FdtNode::PropNode(_, data, size)) = node_iterator
            .skip_while(|node| {
                // TODO? Cheating a bit...
                !node.starts_with("memory@")
            })
            .find(|node| node.has_prop_or_label("reg"))
        {
            unsafe { copy_nonoverlapping(data, res.as_mut_ptr() as *mut u8, size) };
        }
        res
    }
}

struct NodeIterator<'a> {
    offset: usize,
    dtb_reader: &'a DtbReader,
}

#[derive(Debug)]
enum FdtNode<'a> {
    BeginNode(&'a str),
    EndNode,
    Nop,
    PropNode(&'a str, *const u8, usize),
}

impl FdtNode<'_> {
    pub fn has_prop_or_label(&self, text: &str) -> bool {
        match self {
            FdtNode::BeginNode(node_text) => node_text == &text,
            FdtNode::PropNode(node_text, _, _) => node_text == &text,
            _ => false,
        }
    }

    pub fn starts_with(&self, text: &str) -> bool {
        match self {
            FdtNode::BeginNode(node_text) => node_text.starts_with(text),
            FdtNode::PropNode(node_text, _, _) => node_text.starts_with(text),
            _ => false,
        }
    }
}

impl<'a> NodeIterator<'a> {
    fn new(dtb_reader: &'a DtbReader) -> Self {
        let offset = dtb_reader.address + dtb_reader.header.off_dt_struct.to_be() as usize;
        Self { offset, dtb_reader }
    }
}

impl<'a> Iterator for NodeIterator<'a> {
    type Item = FdtNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let token_be = unsafe { *(self.offset as *const u32) };
        let token = token_be.to_be() as u8;
        self.offset += 4;
        match token {
            FDT_BEGIN_NODE => {
                let address = self.offset as *const u8;
                let node_name = unsafe { NullTerminatedStr::as_str(address) };
                // Round to upper 4
                self.offset += usize::max((node_name.len() + 4) & !3, 4);
                Some(FdtNode::BeginNode(node_name))
            }
            FDT_END_NODE => Some(FdtNode::EndNode),
            FDT_PROP => {
                let data_len = unsafe { (*(self.offset as *const u32)).to_be() } as usize;
                let string_off = unsafe { (*((self.offset + 4) as *const u32)).to_be() };
                let data_ptr = (self.offset + 8) as *const u8;
                let string_block_offset = self.dtb_reader.address
                    + self.dtb_reader.header.off_dt_strings.to_be() as usize;
                let address = (string_off as usize + string_block_offset) as *const u8;
                let node_name = unsafe { NullTerminatedStr::as_str(address) };
                // Round to upper 4
                self.offset += (data_len + 3 + 8) & !3;
                Some(FdtNode::PropNode(node_name, data_ptr, data_len))
            }
            FDT_NOP => Some(FdtNode::Nop),
            FDT_END => None,
            _ => {
                unreachable!()
            }
        }
    }
}
