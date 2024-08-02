use crate::mmu::SIFIVE_TEST_ADDRESS;

#[cfg(target_arch = "arm")]
#[allow(dead_code)]
pub mod bcm2836;
pub mod dtb;
#[cfg(target_arch = "arm")]
#[allow(dead_code)]
pub mod mini_uart;
#[cfg(target_arch = "riscv64")]
pub mod uart_16550;
#[cfg(target_arch = "arm")]
pub use raspi2b::*;

pub const UART_ADDRESS: usize = 0x1000_0000;
pub mod virtio;

pub fn shutdown() {
    let address = SIFIVE_TEST_ADDRESS as *mut u32;
    unsafe { address.write_volatile(0x5555) };
}

pub type DeviceId = u32;
