//! # Barba OS
//! Sistema operativo de juguete, basado principalmente en el tutorial de
//! [Stephen Marz](https://osblog.stephenmarz.com/)
//!
//! Este sistema operativo corre sobre RISC-V, más precisamente sobre su
//! emulador en QEMU
//! Debido a su uso de assembly sólo está permitido compilarlo en nightly
#![no_main]
#![no_std]
#![allow(dead_code)]
#![feature(custom_test_frameworks, allocator_api)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

mod assembly;
mod cpu;
mod devices;
mod init;
mod mmu;
#[cfg(test)]
mod test;
mod utils;

use crate::devices::shutdown;
use core::ptr::null;

use init::KMAP_TABLE;

static mut DTB_ADDRESS: *const u8 = null();

#[cfg(target_arch = "riscv64")]
#[no_mangle]
extern "C" fn kmain() {
    use crate::assembly::riscv64::wfi;
    use crate::cpu::riscv64::plic;
    println!("\x1b[1m[kmain start]\x1b[0m");
    println!("Setting up interrupts and PLIC...");
    // We lower the threshold wall so our interrupts can jump over it.
    plic::set_threshold(0);
    // VIRTIO = [1..8]
    // UART0 = 10
    // PCIE = [32..35]
    // Enable the UART interrupt.
    plic::enable(10);
    plic::set_priority(10, 1);
    unsafe {
        let table_ptr = &*(KMAP_TABLE);
        println!("map_table_page: {:p}", table_ptr);
    }
    println!("Press any key...");
    unsafe {
        wfi();
    }
    println!("Exit");
    shutdown();
}

#[cfg(target_arch = "arm")]
#[no_mangle]
extern "C" fn kmain() {
    use crate::assembly::armv7a::wfi;
    use crate::assembly::armv7a::{enable_interrupts, init_timer, irq_init, queue_timer};
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
    #[cfg(test)]
    test_main();
    mmu::print_mem_info();
    unsafe {
        println!("Setting timer...");
        irq_init();
        println!("Enable interrupts");
        enable_interrupts();
        init_timer();
        queue_timer();
        println!("Waiting interrupt...");
        wfi();
    }
    println!("Exit");
    shutdown();
}
