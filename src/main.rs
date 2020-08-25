//! # Barba OS
//! Sistema operativo de juguete, basado principalmente en el tutorial de
//! [Stephen Marz](https://osblog.stephenmarz.com/)
//!
//! Este sistema operativo corre sobre RISC-V, más precisamente sobre su
//! emulador en QEMU
//! Debido a su uso de assembly sólo está permitido compilarlo en nightly
#![no_main]
#![no_std]
#![feature(
    global_asm,
    llvm_asm,
    panic_info_message,
    custom_test_frameworks)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod assembly;
mod cpu;
mod devices;
mod handlers;
mod mmu;
mod system;

#[macro_use]
mod macros;

#[cfg(test)]
mod test;

use core::ptr::null_mut;
use cpu::plic;
use cpu::trap::TrapFrame;
use cpu::trap::schedule_mtime_interrupt;
use devices::uart::Uart;
use handlers::abort;
use mmu::page_table::PageTable;
use mmu::map_table::MapTable;
use system::process;


/// Función principal del kernel
static mut KMAP_TABLE: *mut MapTable = null_mut();

#[no_mangle]
extern "C"
fn kinit() -> usize {
    // Inicializo con la dirección de memoria que configuré en virt.lds
    let uart = Uart::new(0x1000_0000);
    uart.init();
    println!("\x1b[1m[kinit]\x1b[0m");
    mmu::print_mem_info();
    PageTable::init();
    #[cfg(test)]
    test_main();
    mmu::print_mem_info();
    let map_table_page = PageTable::zalloc(1).unwrap();
    let map_table;
    unsafe {
        KMAP_TABLE = map_table_page as *mut MapTable;
        map_table = &mut *(KMAP_TABLE);
        map_table.init_map();
        TrapFrame::init(map_table);
        println!("map_table_page: {:p}", KMAP_TABLE);
    }
    map_table.update_satp(0);
    schedule_mtime_interrupt(20);
    cpu::mscratch_read();
    process::init()
}

#[no_mangle]
extern "C"
fn kmain() {
    println!("\x1b[1m[kmain]\x1b[0m");
    mmu::print_mem_info();
    // cpu::mscratch_read(); //-> tira instruction fault
    cpu::sscratch_read();
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
        // lanzo un page fault, pero como el trap handler por ahora sólo
        // avanza 4 bytes, necesito que esté alineado
        //*(0x1234_5678 as *mut u32) = 0xDEADBEEF;
    }
    println!("\x1b[1m<Finish>\x1b[0m");
    abort();
}
