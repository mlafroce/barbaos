//! # Barba OS
//! Sistema operativo de juguete, basado principalmente en el tutorial de
//! [Stephen Marz](https://osblog.stephenmarz.com/)
//!
//! Este sistema operativo corre sobre RISC-V, más precisamente sobre su
//! emulador en QEMU
//! Debido a su uso de assembly sólo está permitido compilarlo en nightly
#![no_main]
#![no_std]
#![feature(custom_test_frameworks)]
#![test_runner(crate::utils::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod assembly;
mod devices;
mod mmu;
mod utils;

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
    #[cfg(test)]
    test_main();
    println!("BarbaOS booting...");
    let dtb_address = unsafe { DTB_ADDRESS };
    let dtb = DtbReader::new(dtb_address).unwrap();
    dtb.print_boot_info();
    let mem_data = dtb.get_memory_info();
    let heap_end = mem_data[0].to_be() + mem_data[1].to_be();
    let heap_size = heap_end - unsafe { HEAP_START };
    unsafe { HEAP_SIZE.store(heap_size, Ordering::Relaxed) };
    mmu::print_mem_info();
    shutdown();
}
