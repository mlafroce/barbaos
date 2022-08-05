//! Importación de los módulos de assembly utilizados
//! Esto esta casi copypasteado del tutorial
//! Es bastante útil para no usar un linkeditor de forma directa
use core::arch::global_asm;
// assembly.rs
// Assembly imports module
// Stephen Marz
// 20 April 2020

// This came from the Rust book documenting global_asm!.
// They show using include_str! with it to
// import a full assembly file, which is what I want here.

#[cfg(target_arch = "riscv64")]
global_asm!(include_str!("asm/riscv/boot.S"));
#[cfg(target_arch = "riscv64")]
global_asm!(include_str!("asm/riscv/trap.S"));
#[cfg(target_arch = "arm")]
global_asm!(include_str!("asm/armv7a/boot.S"));
#[cfg(target_arch = "arm")]
global_asm!(include_str!("asm/armv7a/trap.S"));
