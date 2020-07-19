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

mod test;

use devices::uart::Uart;
#[macro_use]
mod macros;


#[no_mangle]
extern "C"
fn kmain() {
    // Inicializo con la dirección de memoria que configuré en virt.lds
    let uart = Uart::new(0x1000_0000);
    uart.init();
    #[cfg(test)]
    test_main();
    println!("Hello Rust!");
    mmu::print_mem_info();
}
