use crate::assembly::riscv64;
use crate::cpu::riscv64::trap::TrapFrame;
use crate::devices::dtb::DtbReader;
use crate::devices::uart_16550::Uart;
use crate::mmu::map_table::MapTable;
use crate::mmu::riscv64::{PageTable, GLOBAL_PAGE_TABLE};
use crate::mmu::{HEAP_SIZE, HEAP_START};
use crate::system::process;
use crate::{kmain, mmu};
use crate::{print, println};
use alloc::boxed::Box;
use core::arch::asm;
use core::ptr::{null, null_mut};
use core::sync::atomic::Ordering;

pub static mut KMAP_TABLE: *mut MapTable = null_mut();

extern "C" {
    fn m_trap_vector();
}

pub static mut DTB_ADDRESS: *const u8 = null();

#[no_mangle]
unsafe extern "C" fn supervisor_mode_init() -> ! {
    // armo el registro `mstatus` para volver en modo usuario
    // 1 << 7 : Supervisor's previous protection mode -> 1 (SPP=1 [Supervisor]).
    // 1 << 5 : Supervisor's previous interrupt-enable bit -> 1 (SPIE=1 [Enabled]).
    // 1 << 1 : Supervisor's interrupt-enable bit -> 1 después de sret.
    // armamos el estado "previo" que es el que sret va a restaurar
    let status = (0b11 << 11) | (1 << 7) | (1 << 5);
    // Habilito las interrupciones en modo Supervisor
    let interrupts = 0xaaa;
    // mideleg (Machine Interrupt delegate)
    // Las interrupciones, por defecto, elevan el privilegio a nivel M
    // Delegamos las interrupciones al nivel de supervisor
    // (ver configuración de reg `mie` y tabla de `mcause`)
    // 1 << 1: software interrupts supervisor
    // 1 << 5: timer interrupts supervisor
    // 1 << 9: external interrupts supervisor
    let delegate_mask = (1 << 1) | (1 << 5) | (1 << 9);
    riscv64::mstatus_write(status);
    // Llamamos a kmain despues de inicializar
    riscv64::mepc_write(kmain as *const () as usize);
    riscv64::mie_write(interrupts);
    riscv64::mideleg_write(delegate_mask);
    riscv64::stvec_write(m_trap_vector as *const () as usize);
    riscv64::enable_pmp();
    riscv64::sfence_vma();
    // Salimos en modo supervisor!
    riscv64::mret();
    unreachable!();
}

#[no_mangle]
pub unsafe extern "C" fn user_mode_init(process_pc: usize, sp: usize) -> ! {
    // bits[11::12] = 0 -> Usermode
    let status = (1 << 7) | (1 << 5) | (0b01 << 13);
    let interrupts = 0xa0a;
    let delegate_mask = (1 << 1) | (1 << 5) | (1 << 9);
    riscv64::mstatus_write(status);
    riscv64::mepc_write(process_pc);
    riscv64::mie_write(interrupts);
    riscv64::mideleg_write(delegate_mask);
    riscv64::enable_pmp();
    riscv64::sfence_vma();
    // Salimos en modo Usuario!
    asm!("mv sp, {}", in(reg) sp, options(nomem, nostack));
    riscv64::mret();
    unreachable!();
}

#[no_mangle]
pub extern "C" fn kinit() {
    let dtb_address = unsafe { DTB_ADDRESS };
    let dtb = DtbReader::new(dtb_address).unwrap();
    // Inicializo con la dirección de memoria que configuré en virt.lds
    // TODO: tomar dirección desde DTB
    let uart = Uart::new(0x1000_0000);
    uart.init();
    println!("BarbaOS booting...");
    dtb.print_boot_info();
    let mem_data = dtb.get_memory_info();
    let heap_start = unsafe { HEAP_START };
    let heap_end = mem_data[0].to_be() + mem_data[1].to_be();
    let heap_size = heap_end - heap_start;
    unsafe { HEAP_SIZE.store(heap_size, Ordering::Relaxed) };
    println!("\x1b[1m[kinit]\x1b[0m");
    let mut page_table = PageTable::new(heap_start, heap_size);
    page_table.init();
    if let Some(ptr) = page_table.alloc(1) {
        println!("Alloc success");
        page_table.dealloc(ptr);
    }
    unsafe { GLOBAL_PAGE_TABLE.set_root(&page_table) };
    // TODO: ejecutar tests en módulo
    // #[cfg(test)]
    // test_main();
    mmu::print_mem_info();
    page_table.print_allocations();
    mmu::print_mem_info();
    println!("\x1b[1m<Finish>\x1b[0m");
    #[cfg(test)]
    test_main();
    let mut map_table;
    let page_table = GLOBAL_PAGE_TABLE.get_root();
    map_table = Box::new(MapTable::new(page_table));
    page_table.print_allocations();
    unsafe { map_table.init_map() };
    TrapFrame::init(map_table.as_mut());
    let satp = map_table.get_initial_satp(0);
    unsafe {
        riscv64::satp_write(satp);
        KMAP_TABLE = Box::into_raw(map_table);
    };
    mmu::print_mem_info();
    println!("\x1b[1m<Finish>\x1b[0m");
    process::init(page_table)
}
