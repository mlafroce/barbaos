#[cfg(any(target_arch = "arm", target_arch = "armv7a"))]
pub mod armv7a;
#[cfg(target_arch = "riscv64")]
pub mod riscv64;
