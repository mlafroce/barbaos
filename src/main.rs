//! # Barba OS
//! Sistema operativo de juguete, basado principalmente en el tutorial de
//! [Stephen Marz](https://osblog.stephenmarz.com/)
//!
//! Este sistema operativo corre sobre RISC-V, más precisamente sobre su
//! emulador en QEMU
//! Debido a su uso de assembly sólo está permitido compilarlo en nightly
#![no_main]
#![no_std]
#![feature(custom_test_frameworks, allocator_api)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

mod assembly;
mod devices;
mod mmu;
#[cfg(test)]
mod test;
mod utils;

use crate::mmu::riscv64::{PageTable, GLOBAL_PAGE_TABLE};
use crate::mmu::{HEAP_SIZE, HEAP_START};
use core::ptr::null;
use core::sync::atomic::Ordering;
use devices::dtb::DtbReader;
use devices::shutdown;
use devices::Uart;
use devices::UART_ADDRESS;

static mut DTB_ADDRESS: *const u8 = null();

#[no_mangle]
extern "C" fn kmain() {
    // Inicializo con la dirección de memoria que configuré en virt.lds
    let uart = Uart::new(UART_ADDRESS);
    uart.init();
    println!("BarbaOS booting...");
    let dtb_address = unsafe { DTB_ADDRESS };
    let dtb = DtbReader::new(dtb_address).unwrap();
    dtb.print_boot_info();
    let mem_data = dtb.get_memory_info();
    let heap_end = mem_data[0].to_be() + mem_data[1].to_be();
    let heap_start = unsafe { HEAP_START };
    let heap_size = heap_end - heap_start;
    unsafe { HEAP_SIZE.store(heap_size, Ordering::Relaxed) };
    // TODO: read addresses from dtb
    let mut page_table = PageTable::new(heap_start, heap_size);
    page_table.init();
    unsafe { GLOBAL_PAGE_TABLE.set_root(&page_table) };
    #[cfg(test)]
    test_main();
    mmu::print_mem_info();
    page_table.print_allocations();
    if let Some(page) = page_table.alloc(1) {
        println!("Page allocated: {:?}", page);
        page_table.dealloc(page);
    }
    shutdown();
}
