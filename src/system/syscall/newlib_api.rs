use super::api::Syscall;

extern "C" {
    static errno: isize; // error description
    static end: isize; // sbrk bss end
}

static mut HEAP_END: usize = 0;

#[no_mangle]
pub extern "C" fn _exit(syscall_id: isize) {

}

#[no_mangle]
pub extern "C" fn _close(fd: isize)  -> isize {
    -1
}

#[no_mangle]
pub extern "C" fn _execve(name: *mut u8, argv: *mut *mut u8, env: *mut *mut u8) -> isize {
    let msg = "execve";
    let write_syscall = Syscall::Write {
        fd: 0,
        buf: msg.as_ptr(),
        n_bytes: msg.len(),
    };
    write_syscall.call();
    -1
}

#[no_mangle]
pub extern "C" fn _fork() -> isize {
    -1
}

#[no_mangle]
pub extern "C" fn _fstat(fd: isize) -> isize {
    0
}

#[no_mangle]
pub extern "C" fn _getpid() -> isize {
    1
}

#[no_mangle]
pub extern "C" fn _isatty() -> isize {
    1
}

#[no_mangle]
pub extern "C" fn _kill(pid: isize, sig: isize)  -> isize {
    -1
}

#[no_mangle]
pub extern "C" fn _link(old: *mut u8, new: *mut u8) -> isize {
    -1
}

#[no_mangle]
pub extern "C" fn _lseek(file: isize, ptr: isize, dir: isize) -> isize {
    0
}

#[no_mangle]
pub extern "C" fn _open(name: *const u8, flags: isize, mode: isize) -> isize {
    -1
}

#[no_mangle]
pub extern "C" fn _read(file: isize, ptr: *mut u8, len: isize) -> isize {
    0
}

#[no_mangle]
pub extern "C" fn _sbrk(incr: isize) -> usize {
    let msg = "sbrk";
    let write_syscall = Syscall::Write {
        fd: 0,
        buf: msg.as_ptr(),
        n_bytes: msg.len(),
    };
    write_syscall.call();
    let heap_start = unsafe { &end } as *const _ as *const u8;
    let mut heap_end = unsafe { HEAP_END };
    if heap_end == 0 {
        heap_end = heap_start as usize;
    }
    let prev_heap_end = heap_end;
    heap_end += incr as usize;
    unsafe { HEAP_END = heap_end };
    prev_heap_end
}

#[no_mangle]
pub extern "C" fn _stat(file: *mut u8) -> isize {
    0
}

#[no_mangle]
pub extern "C" fn _times() -> isize {
    0
}

#[no_mangle]
pub extern "C" fn _unlink(syscall_id: isize) -> isize {
    -1
}

#[no_mangle]
pub extern "C" fn _wait(syscall_id: isize) -> isize {
    -1
}

#[no_mangle]
pub extern "C" fn _write(fd: isize, buf: *mut u8, n_bytes: usize) {
    let write_syscall = Syscall::Write {
        fd: fd as usize,
        buf,
        n_bytes,
    };
    write_syscall.call();
}
