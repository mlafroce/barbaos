use core::arch::asm;

pub const REBOOT_MAGIC_1: usize = 0x13011990;
pub const REBOOT_MAGIC_2: usize = 0xCACAFEAA;

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
    #[no_mangle]
    pub extern "C" fn call_arg_0(syscall_id: usize) {
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

    #[no_mangle]
    pub extern "C" fn call_arg_1(syscall_id: usize, arg0: usize) {
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

    #[no_mangle]
    pub extern "C" fn call_arg_2(syscall_id: usize, arg0: usize, arg1: usize) {
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

    #[no_mangle]
    pub extern "C" fn call_arg_3(syscall_id: usize, arg0: usize, arg1: usize, arg2: usize) {
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
