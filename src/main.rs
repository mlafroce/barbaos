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
    panic_info_message)]

pub mod assembly;

/// Override de la macro print de Rust, para imprimir por pantalla, por ahora vacío
#[macro_export]
macro_rules! print
{
    ($($args:tt)+) => ({

    });
}

/// Imprime una linea y un salto de linea al final (o una linea vacía)
#[macro_export]
macro_rules! println
{
    () => ({
        print!("\r\n")
    });
    ($fmt:expr) => ({
        print!(concat!($fmt, "\r\n"))
    });
    ($fmt:expr, $($args:tt)+) => ({
        print!(concat!($fmt, "\r\n"), $($args)+)
    });
}

/// Función utilizada para mecanismos de falla (como el `panic!`)
#[no_mangle]
extern "C" fn eh_personality() {}

/// Esta función es llamada cuando explota todo, como el catch final de
/// un try catch de C++
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("Aborting: ");
    if let Some(p) = info.location() {
        println!(
                    "line {}, file {}: {}",
                    p.line(),
                    p.file(),
                    info.message().unwrap()
        );
    }
    else {
        println!("no information available.");
    }
    abort();
}

/// Una vez que `panic!` imprimió el error, aborto, llamando a la instrucción
/// `wfi`, *Wait for interrupt*
#[no_mangle]
extern "C"
fn abort() -> ! {
    loop {
        unsafe {
            llvm_asm!("wfi"::::"volatile");
        }
    }
}

#[no_mangle]
extern "C"
fn kmain() {
}