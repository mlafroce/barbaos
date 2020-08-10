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
mod devices;
mod handlers;
mod mmu;

#[cfg(test)]
mod test;

use core::ptr::null_mut;
use mmu::page_table::PageTable;
use mmu::map_table::MapTable;
use devices::uart::Uart;
#[macro_use]
mod macros;


/// Función principal del kernel

static mut KMAP_TABLE: *mut MapTable = null_mut();

#[no_mangle]
extern "C"
fn kinit() {
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
    let mut map_table;
    unsafe {
        KMAP_TABLE = map_table_page as *mut MapTable;
        map_table = &mut *(KMAP_TABLE);
        map_table.init_map();
        println!("map_table_page: {:p}", KMAP_TABLE);
    }
    map_table.update_satp();
    unsafe {
        map_table.virt_to_phys(KMAP_TABLE as usize).unwrap();
        map_table.virt_to_phys(&KMAP_TABLE as *const _ as usize).unwrap();
    }
}

#[no_mangle]
extern "C"
fn kmain() {
    println!("\x1b[1m[kmain]\x1b[0m");
    unsafe {
        let test = "kmain";
        let map_table = &*(KMAP_TABLE);
        map_table.test_init_map();
        println!("Hello {}", test);
    }
    mmu::print_mem_info();
    println!("\x1b[1m<Finish>\x1b[0m");
}
