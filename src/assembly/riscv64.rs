use crate::{kmain, DTB_ADDRESS};
use core::arch::{asm, global_asm};

// Assembly imports module
// Stephen Marz
// 20 April 2020

// This came from the Rust book documenting global_asm!.
// They show using include_str! with it to
// import a full assembly file, which is what I want here.
global_asm!(include_str!("../asm/riscv/boot.S"));
global_asm!(include_str!("../asm/riscv/trap.S"));
global_asm!(include_str!("../asm/riscv/mem.S"));

/// # Safety
/// The 'mie' (interrupt enable) register is accessible exclusively in machine mode.
/// Please refer to the privileged ISA documentation for format details.
#[inline]
pub unsafe fn mie_write(value: usize) {
    asm!("csrw mie, {}", in(reg) value, options(nostack))
}

/// # Safety
/// The 'mepc' (exception PC) register is accessible exclusively in machine mode.
/// Value will be used as PC after mret call.
#[inline]
pub unsafe fn mepc_write(value: usize) {
    asm!("csrw mepc, {}", in(reg) value, options(nostack))
}

/// # Safety
/// The 'mscratch' (scratch register) register is accessible exclusively in machine mode.
/// Please refer to the privileged ISA documentation for format details
#[inline]
pub unsafe fn mscratch_write(value: usize) {
    asm!("csrw mscratch, {}", in(reg) value, options(nostack))
}

/// # Safety
/// The 'mstatus' register is accessible exclusively in machine mode.
/// Please refer to the privileged ISA documentation for format details.
#[inline]
pub unsafe fn mstatus_write(value: usize) {
    asm!("csrw mstatus, {}", in(reg) value, options(nostack))
}

/// # Safety
/// The 'mtvec' (trap vector) register is accessible exclusively in machine mode.
/// Must be the address of a trap handler
#[inline]
pub unsafe fn mtvec_write(value: usize) {
    asm!("csrw mtvec, {}", in(reg) value, options(nostack))
}

/// # Safety
/// The 'mret' (return) register is accessible exclusively in machine mode.
/// 'mepc' must be a valid program counter
#[inline]
pub unsafe fn mret() {
    asm!("mret", options(nomem, nostack))
}
#[inline]
pub unsafe fn wfi() {
    asm!("wfi", options(nomem, nostack))
}

extern "C" {
    fn m_trap_vector();
}

#[no_mangle]
extern "C" fn machine_mode_init(_hart_id: usize, dtb_address: *const u8) {
    // Configuramos mstatus: https://ibex-core.readthedocs.io/en/latest/cs_registers.html#machine-status-mstatus
    // Bits 12:11 -> MPP, machine previous privilege. 11 para modo M
    let status = 0b11 << 11;
    // Interrupciones habilitadas:
    // 1 << 3: software interrupts `irq_software_i`
    // 1 << 7: timer interrupts `irq_timer_i`
    // 1 << 11: externa interrupts `irq_extenal_i`
    let interrupts = (1 << 3) | (1 << 7) | (1 << 11);
    // Cargo dirección de memoria de kinit
    let init_addr = kmain as *const () as usize;
    unsafe {
        DTB_ADDRESS = dtb_address;
        mstatus_write(status);
        // Valor de retorno al hacer mret (retorno de excepción)
        mepc_write(init_addr);
        // Configuro la dirección del vector de traps
        mtvec_write(m_trap_vector as *const () as usize);
        // Configuro interrupciones habilitadas
        mie_write(interrupts);
        // mret actualiza `mstatus` y sale de una excepción. En nuestro caso, asigna `mepc` a nuestro program counter
        mret();
    }
}
