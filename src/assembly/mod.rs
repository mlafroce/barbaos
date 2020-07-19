/// Importación de los módulos de assembly utilizados
#[cfg(target_arch = "arm")]
pub mod armv7a;
#[cfg(target_arch = "riscv64")]
pub mod riscv64;
