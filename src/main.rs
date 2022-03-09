//! # Barba OS
//! Sistema operativo de juguete, basado principalmente en el tutorial de
//! [Stephen Marz](https://osblog.stephenmarz.com/)
//!
//! Este sistema operativo corre sobre RISC-V, más precisamente sobre su
//! emulador en QEMU
//! Debido a su uso de assembly sólo está permitido compilarlo en nightly
#![no_main]
#![no_std]
#![allow(dead_code, incomplete_features)]
#![feature(
    custom_test_frameworks,
    allocator_api,
    adt_const_params,
    generic_const_exprs
)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

mod assembly;
mod boot;
mod cpu;
mod devices;
mod filesystem;
mod init;
mod mmu;
mod system;

#[cfg(test)]
mod test;
mod utils;

use core::ptr::null;
use devices::shutdown;

use crate::boot::load_disk;
use crate::devices::virtio::common::DeviceManager;

static mut DTB_ADDRESS: *const u8 = null();

#[cfg(target_arch = "riscv64")]
#[no_mangle]
extern "C" fn kmain() {
    use crate::assembly::riscv64::wfi;
    use crate::cpu::riscv64::plic;
    println!("\x1b[1m[kmain start]\x1b[0m");
    mmu::print_mem_info();
    //mscratch_read(); //-> tira instruction fault
    //sscratch_read(); // OK porque estoy en supervisor
    println!("Setting up interrupts and PLIC...");
    // We lower the threshold wall so our interrupts can jump over it.
    plic::set_threshold(0);
    // VIRTIO = [1..8]
    // UART0 = 10
    // PCIE = [32..35]
    // Enable the UART interrupt.
    for i in 1..=10 {
        plic::enable(i);
        plic::set_priority(i, 1);
    }
    DeviceManager::init();
    load_disk().unwrap();
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
