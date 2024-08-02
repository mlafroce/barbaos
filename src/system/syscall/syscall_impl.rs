use crate::cpu::riscv64::trap::TrapFrame;
use crate::devices::shutdown;
use crate::print;
use crate::system::process::InitProcess;
use crate::system::syscall;
use crate::system::syscall::{REBOOT_MAGIC_1, REBOOT_MAGIC_2};

const ARG_CODE: usize = 10;
const ARG_1: usize = 11;
const ARG_2: usize = 12;
const ARG_3: usize = 13;

/// Ejecuta las distintas syscalls y almacena los datos en el frame del llamador
pub fn execute_syscall(frame: &mut TrapFrame, _epc: usize) {
    let code = frame.regs[ARG_CODE];
    match code {
        syscall::SYS_WRITE => {
            let buf_virt_ptr = frame.regs[ARG_2];
            let buf_size = frame.regs[ARG_3];
            // El Map table lo voy a sacar del proceso cuando tenga un buscador de procesos
            let process = InitProcess::get_process();
            let process_table = &*process.root;
            let buf_phys_ptr = process_table.virt_to_phys(buf_virt_ptr).unwrap() as *const u8;
            for i in 0..buf_size as isize {
                print!("{}", unsafe { *buf_phys_ptr.offset(i) } as char);
            }
        }
        syscall::SYS_REBOOT => {
            if frame.regs[ARG_1] == REBOOT_MAGIC_1 && frame.regs[ARG_2] == REBOOT_MAGIC_2 {
                shutdown();
            }
        }
        syscall::SYS_POPMSGBOX => {}
        _ => {
            unimplemented!()
        }
    }
}
