use crate::{print, println};
use crate::devices::uart::Uart;
use crate::cpu::trap::TrapFrame;
use crate::MapTable;

use crate::process::INIT_PROCESS;

/// Esta macro recibe un id de syscall y una cantidad variable de argumentos
/// luego llama a call_arg_n según la cantidad que posee
macro_rules! syscalls {
(enum $ename:ident {
        $($vname:ident ( $syscall_id: expr, $($varg: ident: $vty: ty),* )),*
    }) => {
        pub enum $ename {
            $($vname { $($varg: $vty),* }),*
        }

        impl $ename {
            pub fn call(&self) {
                match self {
                    $($ename::$vname{$($varg),*} => syscalls!(@call ($syscall_id, $($varg)*))),*
                }
            }
        }
    };
    (@call ($syscall_id:expr)) => {Syscall::call_arg_0($syscall_id)};
    (@call ($syscall_id:expr, $a:tt)) => {Syscall::call_arg_1($syscall_id, *$a as usize)};
    (@call ($syscall_id:expr, $a:tt $b:tt)) => {Syscall::call_arg_2($syscall_id, *$a as usize, *$b as usize)};
    (@call ($syscall_id:expr, $a:tt $b:tt $c:tt)) => {Syscall::call_arg_3($syscall_id, *$a as usize, *$b as usize, *$c as usize)};
}

syscalls! {
    enum Syscall {
        Write(0, fd: usize, buf: *const u8, n_bytes: usize)
    }
}

impl Syscall {
    pub fn call_arg_0(syscall_id: usize) {
        // ¿Esto es trivial?
        unsafe {
            llvm_asm!("mv a0, $0" :: "r"(syscall_id));
            llvm_asm!("ecall");
        }
    }

    pub fn call_arg_1(syscall_id: usize, arg0: usize) {
        // ¿Esto es trivial?
        unsafe {
            llvm_asm!("mv a0, $0" :: "r"(syscall_id));
            llvm_asm!("mv a1, $0" :: "r"(arg0));
            llvm_asm!("ecall");

        }
    }

    pub fn call_arg_2(syscall_id: usize, arg0: usize, arg1: usize) {
        // ¿Esto es trivial?
        unsafe {
            llvm_asm!("mv a0, $0" :: "r"(syscall_id));
            llvm_asm!("mv a1, $0" :: "r"(arg0));
            llvm_asm!("mv a2, $0" :: "r"(arg1));
            llvm_asm!("ecall");
        }
    }

    pub fn call_arg_3(syscall_id: usize, arg0: usize, arg1: usize, arg2: usize) {
        // ¿Esto es trivial?
        unsafe {
            llvm_asm!("mv a0, $0" :: "r"(syscall_id));
            llvm_asm!("mv a1, $0" :: "r"(arg0));
            llvm_asm!("mv a2, $0" :: "r"(arg1));
            llvm_asm!("mv a3, $0" :: "r"(arg2));
            llvm_asm!("ecall");
        }
    }
    // TODO: los que faltan
}

pub fn call_syscall(syscall: &Syscall) {
    syscall.call();
}

/// Ejecuta las distintas syscalls y almacena los datos en el frame del llamador
pub fn execute_syscall(frame: &mut TrapFrame, epc: usize) {
    let code = frame.regs[10];
    match code {
        0 => {
            let buf_virt_ptr = frame.regs[12];
            let buf_size = frame.regs[13];
            // El Map table lo voy a sacar del proceso cuando tenga un buscador de procesos
            let process_table;
            unsafe {
                let process = &INIT_PROCESS.as_ref().unwrap();
                process_table = &*process.root;
            }
            println!("buf virt ptr: {:x}", buf_virt_ptr);

            let buf_phys_ptr = process_table.virt_to_phys(buf_virt_ptr).unwrap();
            let buf_str: &str;
            unsafe {
                let buf_u8 = core::slice::from_raw_parts(buf_phys_ptr as *const u8, buf_size);
                buf_str = core::str::from_utf8_unchecked(buf_u8);
            }
            println!("{:?}", buf_str);
        }
        _ => {
            unimplemented!()
        }
    }
}