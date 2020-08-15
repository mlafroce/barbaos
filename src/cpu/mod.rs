pub mod trap;

pub fn mscratch_write(val: usize) {
    unsafe {
        llvm_asm!("csrw mscratch, $0" ::"r"(val));
    }
}

pub fn mscratch_read() -> usize {
    unsafe {
        let rval;
        llvm_asm!("csrr $0, mscratch" : "=r"(rval));
        rval
    }
}

pub fn sscratch_read() -> usize {
    unsafe {
        let rval;
        llvm_asm!("csrr $0, sscratch" : "=r"(rval));
        rval
    }
}

pub fn sscratch_write(val: usize) {
    unsafe {
        llvm_asm!("csrw sscratch, $0" ::"r"(val));
    }
}

pub fn satp_fence_asid(asid: usize) {
    unsafe {
        llvm_asm!("sfence.vma zero, $0" :: "r"(asid));
    }
}
