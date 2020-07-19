use core::arch::global_asm;

global_asm!(include_str!("../asm/armv7a/boot.S"));
global_asm!(include_str!("../asm/armv7a/trap.S"));
