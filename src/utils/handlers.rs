use crate::{print, println};
use core::arch::asm;

/// Función utilizada para mecanismos de falla (como el `panic!`)
#[no_mangle]
extern "C" fn eh_personality() {}

/// Esta función es llamada cuando explota todo, como el catch final de
/// un try catch de C++
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("Aborting: ");
    if let Some(p) = info.location() {
        println!("line {}, file {}: {}", p.line(), p.file(), info.message());
    } else {
        println!("no information available.");
    }
    abort();
}

/// Una vez que `panic!` imprimió el error, aborto, llamando a la instrucción
/// `wfi`, *Wait for interrupt*
#[no_mangle]
pub extern "C" fn abort() -> ! {
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}
