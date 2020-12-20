pub mod uart_16550;
#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
pub use raspi2b::*;
#[cfg(target_arch = "riscv64")]
pub use riscv::*;

#[cfg(target_arch = "riscv64")]
pub mod riscv {
    pub const UART_ADDRESS: usize = 0x1000_0000;

    pub fn shutdown() {
        let address = 0x100000 as *mut u32;
        unsafe { address.write_volatile(0x5555) };
    }
}

/// Using bcm2835_wdt mechanism from
/// https://github.com/intel/idxd-driver/blob/master/drivers/watchdog/bcm2835_wdt.c
#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
pub mod raspi2b {
    pub const UART_ADDRESS: usize = 0x3f20_1000;
    const WATCHDOG_ADDRESS: usize = 0x3f10_0000;
    const PM_RSTC: usize = 0x1C;
    const PM_RSTS: usize = 0x20;
    const PM_WDOG: usize = 0x24;
    // Prevent accidental writes
    const PM_PASSWORD: u32 = 0x5a00_0000;
    const PM_RSTC_WRCFG_CLR: u32 = 0xffff_ffcf;
    const PM_RSTC_WRCFG_FULL_RESET: u32 = 0x20;
    const PM_RSTS_RASPBERRYPI_HALT: u32 = 0x555;

    pub fn shutdown() {
        let wd_address = (WATCHDOG_ADDRESS + PM_RSTS) as *mut u32;
        let mut val = unsafe { wd_address.read_volatile() };
        val |= PM_PASSWORD | PM_RSTS_RASPBERRYPI_HALT;
        unsafe {
            wd_address.write_volatile(val);
        }

        /* Continue with normal reset mechanism */
        restart();
    }

    pub fn restart() {
        let wdog_address = (WATCHDOG_ADDRESS + PM_WDOG) as *mut u32;
        let rstc_address = (WATCHDOG_ADDRESS + PM_RSTC) as *mut u32;
        unsafe {
            wdog_address.write_volatile(10 | PM_PASSWORD);
        }
        let mut val = unsafe { rstc_address.read_volatile() };
        val &= PM_RSTC_WRCFG_CLR;
        val |= PM_PASSWORD | PM_RSTC_WRCFG_FULL_RESET;
        unsafe {
            rstc_address.write_volatile(val);
        }
    }
}
