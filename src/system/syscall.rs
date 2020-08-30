use crate::cpu::riscv64::trap::TrapFrame;
use crate::devices::shutdown;
use crate::system::process::InitProcess;
use crate::{print, println};
use core::arch::asm;

pub const REBOOT_MAGIC_1: usize = 0x13011990;
pub const REBOOT_MAGIC_2: usize = 0xCACAFEAA;
const ARG_1: usize = 11;
const ARG_2: usize = 12;
const ARG_3: usize = 13;

pub const SYS_WRITE: usize = 1;
pub const SYS_REBOOT: usize = 48;

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
    (@call ($syscall_id:expr, $a:tt $b:tt $c:tt)) => {Syscall::call_arg_3($syscall_id, *$a, *$b as usize, *$c as usize)};
}

syscalls! {
    enum Syscall {
        Write(SYS_WRITE, fd: usize, buf: *const u8, n_bytes: usize),
        Reboot(SYS_REBOOT, magic1: usize, magic2: usize, poweroff: bool)
    }
}

impl Syscall {
    pub fn call_arg_0(syscall_id: usize) {
        // ¿Esto es trivial?
        unsafe {
            asm!(
            "mv a0, {}",
            "ecall",
            in(reg) syscall_id,
            out("a0") _
            );
        }
    }

    pub fn call_arg_1(syscall_id: usize, arg0: usize) {
        // ¿Esto es trivial?
        unsafe {
            asm!(
            "mv a0, {}",
            "mv a1, {}",
            "ecall",
            in(reg) syscall_id,
            in(reg) arg0,
            out("a0") _,
            out("a1") _
            );
        }
    }

    pub fn call_arg_2(syscall_id: usize, arg0: usize, arg1: usize) {
        // ¿Esto es trivial?
        unsafe {
            asm!(
            "mv a0, {}",
            "mv a1, {}",
            "mv a2, {}",
            "ecall",
            in(reg) syscall_id,
            in(reg) arg0,
            in(reg) arg1,
            out("a0") _,
            out("a1") _,
            out("a2") _
            );
        }
    }

    pub fn call_arg_3(syscall_id: usize, arg0: usize, arg1: usize, arg2: usize) {
        // ¿Esto es trivial?
        unsafe {
            asm!(
            "mv a0, {}",
            "mv a1, {}",
            "mv a2, {}",
            "mv a3, {}",
            "ecall",
            in(reg) syscall_id,
            in(reg) arg0,
            in(reg) arg1,
            in(reg) arg2,
            out("a0") _,
            out("a1") _,
            out("a2") _,
            out("a3") _
            );
        }
    }
    // TODO: los que faltan
}

pub fn call_syscall(syscall: &Syscall) {
    syscall.call();
}

/// Ejecuta las distintas syscalls y almacena los datos en el frame del llamador
pub fn execute_syscall(frame: &mut TrapFrame, _epc: usize) {
    let code = frame.regs[10];
    println!("Code: {}", code);
    match code {
        SYS_WRITE => {
            let buf_virt_ptr = frame.regs[ARG_2];
            let buf_size = frame.regs[ARG_3];
            // El Map table lo voy a sacar del proceso cuando tenga un buscador de procesos
            let process = InitProcess::get_process();
            let process_table = &process.root;
            println!("Trying to access {:x}", buf_virt_ptr);
            let buf_phys_ptr = process_table.virt_to_phys(buf_virt_ptr).unwrap();
            let buf_str: &str;
            unsafe {
                let buf_u8 = core::slice::from_raw_parts(buf_phys_ptr as *const u8, buf_size);
                buf_str = core::str::from_utf8_unchecked(buf_u8);
            }
            println!("{:?}", buf_str);
        }
        SYS_REBOOT => {
            if frame.regs[ARG_1] == REBOOT_MAGIC_1 && frame.regs[ARG_2] == REBOOT_MAGIC_2 {
                shutdown();
            }
        }
        _ => {
            unimplemented!()
        }
    }
}

pub fn call_arg_3(syscall_id: usize, arg0: usize, arg1: usize, arg2: usize) {
    // ¿Esto es trivial?
    unsafe {
        asm!(
        "1: nop",
        "j 1b",
        "mv a0, {}",
        "mv a1, {}",
        "mv a2, {}",
        "mv a3, {}",
        "ecall",
        in(reg) syscall_id,
        in(reg) arg0,
        in(reg) arg1,
        in(reg) arg2,
        out("a0") _,
        out("a1") _,
        out("a2") _,
        out("a3") _
        );
    }
}
