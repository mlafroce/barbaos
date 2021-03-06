//! # Módulo de MMU (Memory management unit)
//!
//! La MMU de RISCV tiene muchas particularidades. Para empezar, la memoria
//! virtual está disponible en los modos supervisor y usuario, dado que el modo
//! máquina sólo accede a la memoria física de forma directa.
//! Además, si bien la versión de 32 bits de la arquitectura tiene una cantidad
//! de bits fija para la memoria virtual, en la versión de 64 bits esta
//! cantidad tiene 3 configuracíones distintas.
//!
//! La memoria virtual se implementa con una tabla de paginación que funciona
//! como raiz de un árbol de páginas. Puede haber hasta 3 (o 4 según la
//! configuración elegida) niveles de jerarquía. Cualquier tabla de árbol
//! puede ser una hoja, lo que nos permite reservar páginas de memoria de
//! distinto tamaño.
//!
//! La implementación de una MMU sólo necesita de un "walker" que recorra las
//! páginas de memoria (PTW), y de un *Translation look-aside buffer* TLB.
use crate::{print, println};
use crate::devices::uart::Uart;

pub mod page_table;
pub mod map_table;

extern "C" {
    static HEAP_START: usize;
    static HEAP_SIZE: usize;
    static TEXT_START: usize;
    static TEXT_END: usize;
    static DATA_START: usize;
    static DATA_END: usize;
    static RODATA_START: usize;
    static RODATA_END: usize;
    static BSS_START: usize;
    static BSS_END: usize;
    static KERNEL_STACK_START: usize;
    static KERNEL_STACK_END: usize;
}

pub const MTIME_ADDRESS: usize =    0x0200_bff8;
pub const MTIMECMP_ADDRESS: usize = 0x0200_4000;

/// Constantes con direcciones de regiones importantes de memoria
pub fn print_mem_info() {
    unsafe {
        println!("\x1b[1m[print_mem_info]\x1b[0m");
        println!("Static variables:   \x1b[1m{:p}\x1b[0m", &TEXT_START);
        println!("Text start:         \x1b[1m{:#x}\x1b[0m", TEXT_START);
        println!("Text end:           \x1b[1m{:#x}\x1b[0m", TEXT_END);
        println!("RO Data start:      \x1b[1m{:#x}\x1b[0m", RODATA_START);
        println!("RO Data end:        \x1b[1m{:#x}\x1b[0m", RODATA_END);
        println!("Data start:         \x1b[1m{:#x}\x1b[0m", DATA_START);
        println!("Data end:           \x1b[1m{:#x}\x1b[0m", DATA_END);
        println!("BSS start:          \x1b[1m{:#x}\x1b[0m", BSS_START);
        println!("BSS end:            \x1b[1m{:#x}\x1b[0m", BSS_END);
        println!("Kernel stack start: \x1b[1m{:#x}\x1b[0m", KERNEL_STACK_START);
        println!("Kernel stack end:   \x1b[1m{:#x}\x1b[0m", KERNEL_STACK_END);
        println!("Heap start:         \x1b[1m{:#x}\x1b[0m", HEAP_START);
        println!("Heap size:          \x1b[1m{:10}\x1b[0m bytes", HEAP_SIZE);
    }
}
