use crate::{kmain, DTB_ADDRESS};
use core::arch::{asm, global_asm};

global_asm!(include_str!("../asm/armv7a/boot.S"));
global_asm!(include_str!("../asm/armv7a/trap.S"));
global_asm!(include_str!("../asm/armv7a/mem.S"));

#[inline]
pub unsafe fn wfi() {
    asm!("wfi", options(nomem, nostack))
}

#[no_mangle]
extern "C" fn machine_init(_zero: usize, _hart_id: usize, dtb_address: *const u8) {
    // Cargo dirección de memoria de kinit
    unsafe {
        DTB_ADDRESS = dtb_address;
    }
    kmain();
}
