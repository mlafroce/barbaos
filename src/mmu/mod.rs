//! # Módulo de MMU (Memory management unit)
//!
//! Por el momento sólo implementamos la parte de paginación
use crate::{print, println};
use core::sync::atomic::{AtomicUsize, Ordering};

pub mod map_table;
pub mod riscv64;

extern "C" {
    pub(crate) static TEXT_START: usize;
    pub(crate) static TEXT_END: usize;
    static DATA_START: usize;
    static DATA_END: usize;
    static RODATA_START: usize;
    pub(crate) static RODATA_END: usize;
    static BSS_START: usize;
    static BSS_END: usize;
    static KERNEL_STACK_START: usize;
    static KERNEL_STACK_END: usize;
    pub static HEAP_START: usize;
    pub static HEAP_SIZE: AtomicUsize;
}

pub const MTIME_ADDRESS: usize = 0x0200_bff8;
pub const MTIMECMP_ADDRESS: usize = 0x0200_4000;
pub const SIFIVE_TEST_ADDRESS: usize = 0x0010_0000;
pub const VIRTIO_ADDRESS: usize = 0x1000_1000;

/// Constantes con direcciones de regiones importantes de memoria
pub fn print_mem_info() {
    unsafe {
        println!("\x1b[1m[print_mem_info]\x1b[0m");
        println!("Static variables:   \x1b[1m{:#x}\x1b[0m", &TEXT_START);
        println!("Text start:         \x1b[1m{:#x}\x1b[0m", TEXT_START);
        println!("Text end:           \x1b[1m{:#x}\x1b[0m", TEXT_END);
        println!("RO Data start:      \x1b[1m{:#x}\x1b[0m", RODATA_START);
        println!("RO Data end:        \x1b[1m{:#x}\x1b[0m", RODATA_END);
        println!("Data start:         \x1b[1m{:#x}\x1b[0m", DATA_START);
        println!("Data end:           \x1b[1m{:#x}\x1b[0m", DATA_END);
        println!("BSS start:          \x1b[1m{:#x}\x1b[0m", BSS_START);
        println!("BSS end:            \x1b[1m{:#x}\x1b[0m", BSS_END);
        println!(
            "Kernel stack start: \x1b[1m{:#x}\x1b[0m",
            KERNEL_STACK_START
        );
        println!("Kernel stack end:   \x1b[1m{:#x}\x1b[0m", KERNEL_STACK_END);
        println!("Heap start:         \x1b[1m{:#x}\x1b[0m", HEAP_START);
        println!(
            "Heap size:          \x1b[1m{:#x}\x1b[0m bytes",
            HEAP_SIZE.load(Ordering::Relaxed)
        );
    }
}
