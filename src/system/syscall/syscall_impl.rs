use crate::cpu::riscv64::trap::TrapFrame;
use crate::devices::shutdown;
use crate::system::process::INIT_PROCESS;
use crate::system::syscall;
use crate::system::syscall::{REBOOT_MAGIC_1, REBOOT_MAGIC_2};
use crate::{print, println};

const ARG_1: usize = 11;
const ARG_2: usize = 12;
const ARG_3: usize = 13;

/// Ejecuta las distintas syscalls y almacena los datos en el frame del llamador
pub fn execute_syscall(frame: &mut TrapFrame, _epc: usize) {
    let code = frame.regs[10];
    match code {
        syscall::SYS_WRITE => {
            let buf_virt_ptr = frame.regs[ARG_2];
            let buf_size = frame.regs[ARG_3];
            // El Map table lo voy a sacar del proceso cuando tenga un buscador de procesos
            let process_table;
            unsafe {
                let process = INIT_PROCESS.as_ref().unwrap();
                process_table = &*process.root;
            }
            let buf_phys_ptr = process_table.virt_to_phys(buf_virt_ptr).unwrap() as *const u8;
            for i in 0..buf_size as isize {
                print!("{}", unsafe { *buf_phys_ptr.offset(i) } as char);
            }
        }
        syscall::SYS_BRK => {
            let heap_end = frame.regs[ARG_1];
            println!("Heap end: {:x}", heap_end);
        }
        syscall::SYS_REBOOT => {
            if frame.regs[ARG_1] == REBOOT_MAGIC_1 && frame.regs[ARG_2] == REBOOT_MAGIC_2 {
                shutdown();
            }
        }
        _ => {
            unimplemented!()
        }
    }
}
