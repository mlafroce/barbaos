use core::ptr::null_mut;
use core::mem::size_of;
use crate::cpu::plic;
use crate::devices::uart::{Uart, read_uart};
use crate::mmu::page_table::{PageTable, PAGE_SIZE};
use crate::mmu::map_table::MapTable;
use crate::mmu::map_table::EntryBits;
use crate::mmu::{MTIME_ADDRESS, MTIMECMP_ADDRESS};
use crate::{print, println};

/// Trap Frames para cada núcleo (8 núcleos en total)
pub static mut KERNEL_TRAP_FRAME: [TrapFrame; 8] =
    [TrapFrame::new(); 8];

pub const TIMER_OFFSET_VALUE: u64 = 1000;
const MSECS_CYCLES: u64 = 10_000;

const UART_INT: u32 = 10;

#[repr(C)]
#[derive(Clone, Copy)]
/// # Trap Frame
/// Representa los registros almacenados cada vez que caemos en el `asm_trap_vector`
/// De esta forma, cada vez que ocurre un trap, guardamos el estado de la CPU, realizamos
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
        TrapFrame { regs: [0; 32],
                    fregs: [0; 32],
                    satp: 0,
                    trap_stack: null_mut(),
                     hartid: 0 }
    }

    /// inicializa el trap frame del hart 0
    pub fn init(map_table: &mut MapTable) {
        let satp = map_table.get_initial_satp(0);
        let frame;
        unsafe {
            frame = &mut KERNEL_TRAP_FRAME[0];
        }
        // El contexto de mi modo máquina y de supervisor es el mismo
        // (Obviamente esto es un horror de seguridad)
        let scratch_val = frame as *const TrapFrame as usize;
        // Mi scratch tanto en supervisor como en máquina es el trapframe 0 global del kernel
        super::mscratch_write(scratch_val);
        super::sscratch_write(scratch_val);
        frame.satp = satp;
        let trap_stack = PageTable::zalloc(1).unwrap();
        unsafe {
            // Reservo memoria para el stack de mi trap handler
            // Como el stack crece de arriba hacia abajo, le paso la dirección del final del stack
            frame.trap_stack = trap_stack.add(PAGE_SIZE);
            map_table.range_map(scratch_val, scratch_val + size_of::<TrapFrame>(), EntryBits::ReadWrite.val());
            // Mapeo trap_stack
            map_table.range_map(trap_stack as usize, trap_stack.add(PAGE_SIZE) as usize, EntryBits::ReadWrite.val());
        }
        // sincronizo memoria
        super::satp_fence_asid(0);
    }
}

#[no_mangle]
extern "C" fn m_trap_handler(epc: usize,
                     tval: usize,
                     cause: usize,
                     hart: usize,
                     status: usize,
                     frame: &mut TrapFrame)
                     -> usize {
    // We're going to handle all traps in machine mode. RISC-V lets
    // us delegate to supervisor mode, but switching out SATP (virtual memory)
    // gets hairy.
    let is_async =  cause >> 63 & 1 == 1;
    // The cause contains the type of trap (sync, async) as well as the cause
    // number. So, here we narrow down just the cause number.
    let cause_num = cause & 0xfff;
    let mut return_pc = epc;
    if is_async {
        // Asynchronous trap
        match cause_num {
            3 => {
                // Machine software
                println!("Machine software interrupt CPU#{}", hart);
            },
            7 => {
                // Machine timer
                schedule_mtime_interrupt(TIMER_OFFSET_VALUE);
            },
            11 => {
                // Machine external interrupt
                if let Some(interrupt) = plic::next_interrupt() {
                    // Ocurrió una interrupción en el Claim register
                    match interrupt {
                        UART_INT => {
                            let uart = Uart::new(0x1000_0000);
                            read_uart(&uart);
                        },
                        _ => {
                            println!("Unknown interrupt: {}", interrupt);
                        },
                    }
                    plic::complete(interrupt);
                }
            },
            _ => {
                panic!("Unhandled async trap CPU#{} -> {}\n", hart, cause_num);
            }
        }
    }
    else {
        // Synchronous trap
        match cause_num {
            2 => {
                // Illegal instruction
                panic!("Illegal instruction CPU#{} -> 0x{:08x}: 0x{:08x}\n", hart, epc, tval);
            },
            7 => {
                // Illegal instruction
                panic!("Store/AMO Access Fault#{} -> 0x{:08x}: 0x{:08x}\n", hart, epc, tval);
            },
            8 => {
                // Environment (system) call from User mode
                println!("E-call from User mode! CPU#{} -> 0x{:08x}", hart, epc);
                return_pc += 4;
            },
            9 => {
                // Environment (system) call from Supervisor mode
                println!("E-call from Supervisor mode! CPU#{} -> 0x{:08x}", hart, epc);
                return_pc += 4;
            },
            11 => {
                // Environment (system) call from Machine mode
                panic!("E-call from Machine mode! CPU#{} -> 0x{:08x}\n", hart, epc);
            },
            // Page faults
            12 => {
                // Instruction page fault
                panic!("Instruction page fault CPU#{} -> 0x{:08x}: 0x{:08x}", hart, epc, tval);
                //return_pc += 4;
            },
            13 => {
                // Load page fault
                println!("Load page fault CPU#{} -> 0x{:08x}: 0x{:08x}", hart, epc, tval);
                return_pc += 4;
            },
            15 => {
                // Store page fault
                println!("Store page fault CPU#{} -> 0x{:08x}: 0x{:08x}", hart, epc, tval);
                return_pc += 4;
            },
            _ => {
                panic!("Unhandled sync trap CPU#{} -> {}\n", hart, cause_num);
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
