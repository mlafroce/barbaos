//! # Barba OS
//! Sistema operativo de juguete, basado principalmente en el tutorial de
//! [Stephen Marz](https://osblog.stephenmarz.com/)
//!
//! Este sistema operativo corre sobre RISC-V, más precisamente sobre su
//! emulador en QEMU
//! Debido a su uso de assembly sólo está permitido compilarlo en nightly
#![no_main]
#![no_std]

pub mod assembly;
pub mod devices;
pub mod utils;

use devices::shutdown;
use devices::uart_16550::Uart;

#[no_mangle]
extern "C" fn kmain() {
    // Inicializo con la dirección de memoria que configuré en virt.lds
    let uart = Uart::new(0x1000_0000);
    uart.init();
    println!("Hello Rust!");
    shutdown();
}
