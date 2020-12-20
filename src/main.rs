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
mod utils;

use devices::shutdown;
use devices::uart_16550::Uart;
use devices::UART_ADDRESS;

#[no_mangle]
extern "C" fn kmain() {
    // Inicializo con la dirección de memoria que configuré en virt.lds
    let uart = Uart::new(UART_ADDRESS);
    uart.init();
    #[cfg(test)]
    test_main();
    println!("Hello Rust!");
    shutdown();
}
