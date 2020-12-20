use core::arch::{asm, global_asm};

global_asm!(include_str!("../asm/armv7a/boot.S"));
global_asm!(include_str!("../asm/armv7a/trap.S"));

#[inline]
pub unsafe fn wfi() {
    asm!("wfi", options(nomem, nostack))
}
