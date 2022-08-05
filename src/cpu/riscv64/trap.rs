use crate::assembly::riscv64;
use crate::cpu::riscv64::plic;
use crate::devices::uart_16550::{read_uart, Uart};
use crate::mmu::map_table::{EntryBits, MapTable};
use crate::mmu::riscv64::{MTIMECMP_ADDRESS, MTIME_ADDRESS, PAGE_SIZE};
use crate::system::syscall::syscall_impl::execute_syscall;
use crate::{print, println};
use alloc::boxed::Box;
use core::mem::size_of;
use core::ptr::null_mut;

/// Trap Frames para cada núcleo (8 núcleos en total)
/// TODO: Fix!
pub static mut KERNEL_TRAP_FRAME: [TrapFrame; 8] = [TrapFrame::new(); 8];

pub const TIMER_OFFSET_VALUE: u64 = 1000;
const MSECS_CYCLES: u64 = 10_000;

const UART_INT: u32 = 10;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
/// # Trap Frame
/// Representa los registros almacenados cada vez que caemos en el `asm_trap_vector`
/// De esta forma, cada vez que ocurre un riscv64, guardamos el estado de la CPU, realizamos
/// nuestras tareas, y restauramos el estado de la cpu.
pub struct TrapFrame {
    pub regs: [usize; 32],
    pub fregs: [usize; 32],
    pub satp: usize,
    pub trap_stack: *mut u8,
    pub hartid: usize,
}

impl TrapFrame {
    /// Devuelve un TrapFrame inicializado en 0
    pub const fn new() -> Self {
        TrapFrame {
            regs: [0; 32],
            fregs: [0; 32],
            satp: 0,
            trap_stack: null_mut(),
            hartid: 0,
        }
    }

    /// inicializa el trap frame del hart 0
    pub fn init(map_table: &mut MapTable) {
        let _satp = map_table.get_initial_satp(0);
        let frame;
        unsafe {
            frame = &mut KERNEL_TRAP_FRAME[0];
        }
        frame.satp = map_table.get_initial_satp(0);
        // El scratch apunta al contexto de mi frame
        let scratch_val = frame as *const TrapFrame as usize;
        unsafe { riscv64::mscratch_write(scratch_val) };
        let trap_stack_mem = Box::<u8>::new(0);
        let trap_stack = Box::<u8>::into_raw(trap_stack_mem);
        unsafe {
            // Reservo memoria para el stack de mi riscv64 handler
            // Como el stack crece de arriba hacia abajo, le paso la dirección del final del stack
            frame.trap_stack = trap_stack.add(PAGE_SIZE);
            map_table.range_map(
                scratch_val,
                scratch_val + size_of::<TrapFrame>(),
                EntryBits::ReadWrite.val(),
            );
        }
        // sincronizo memoria
        unsafe { riscv64::satp_fence_asid(0) };
    }
}

#[no_mangle]
extern "C" fn m_trap_handler(
    epc: usize,
    tval: usize,
    cause: usize,
    hart: usize,
    _status: usize,
    frame: &mut TrapFrame,
) -> usize {
    let is_async = cause >> 63 & 1 == 1;
    // The cause contains the type of riscv64 (sync, async) as well as the cause
    // number. So, here we narrow down just the cause number.
    let cause_num = cause & 0xfff;
    let mut return_pc = epc;
    if is_async {
        // Asynchronous riscv64
        match cause_num {
            3 => {
                // Machine software
                println!("Machine software interrupt CPU#{}", hart);
            }
            7 => {
                // Machine timer
                schedule_mtime_interrupt(TIMER_OFFSET_VALUE);
            }
            11 => {
                // Machine external interrupt
                if let Some(interrupt) = plic::next_interrupt() {
                    // Ocurrió una interrupción en el Claim register
                    match interrupt {
                        1..=8 => {
                            println!("VirtIO interrupt {}", interrupt);
                        }
                        UART_INT => {
                            let uart = Uart::new(0x1000_0000);
                            read_uart(&uart);
                        }
                        _ => {
                            println!("Unknown interrupt: {}", interrupt);
                        }
                    }
                    plic::complete(interrupt);
                }
            }
            _ => {
                panic!("Unhandled async riscv64 CPU#{} -> {}\n", hart, cause_num);
            }
        }
    } else {
        // Synchronous riscv64
        match cause_num {
            1 => {
                // Illegal instruction
                panic!(
                    "Instruction access fault CPU#{} -> 0x{:08x}: 0x{:08x}\n",
                    hart, epc, tval
                );
            }
            2 => {
                // Illegal instruction
                panic!(
                    "Illegal instruction CPU#{} -> 0x{:08x}: 0x{:08x}\n",
                    hart, epc, tval
                );
            }
            5 => {
                //
                panic!(
                    "Load Access Fault CPU#{} -> 0x{:08x}: 0x{:08x}\n",
                    hart, epc, tval
                );
            }
            7 => {
                // Illegal instruction
                panic!(
                    "Store/AMO Access Fault CPU#{} -> 0x{:08x}: 0x{:08x}\n",
                    hart, epc, tval
                );
            }
            8 => {
                // Environment (system) call from User mode
                println!("E-call from User mode! CPU#{} -> 0x{:08x}", hart, epc);
                execute_syscall(frame, epc);
                return_pc += 4;
            }
            9 => {
                // Environment (system) call from Supervisor mode
                println!("E-call from Supervisor mode! CPU#{} -> 0x{:08x}", hart, epc);
                return_pc += 4;
            }
            11 => {
                // Environment (system) call from Machine mode
                panic!("E-call from Machine mode! CPU#{} -> 0x{:08x}\n", hart, epc);
            }
            // Page faults
            12 => {
                // Instruction page fault
                panic!(
                    "Instruction page fault CPU#{} -> 0x{:08x}: 0x{:08x}",
                    hart, epc, tval
                );
                //return_pc += 4;
            }
            13 => {
                // Load page fault
                println!(
                    "Load page fault CPU#{} -> 0x{:08x}: 0x{:08x}",
                    hart, epc, tval
                );
                return_pc += 4;
            }
            15 => {
                // Store page fault
                println!(
                    "Store page fault CPU#{} -> 0x{:08x}: 0x{:08x}",
                    hart, epc, tval
                );
                return_pc += 4;
            }
            _ => {
                panic!("Unhandled sync riscv64 CPU#{} -> {}\n", hart, cause_num);
            }
        }
    };
    // Finally, return the updated program counter
    return_pc
}

/// Asigna un valor al registro `mtimecmp` relativo al tiempo actual
/// Se lanza una interrupcción luego de `msecs` milisegundos
pub fn schedule_mtime_interrupt(msecs: u64) {
    let mtimecmp = MTIMECMP_ADDRESS as *mut u64;
    let mtime = MTIME_ADDRESS as *const u64;
    unsafe {
        let next_interrupt = mtime.read_volatile().wrapping_add(msecs * MSECS_CYCLES);
        mtimecmp.write_volatile(next_interrupt);
    }
}
