pub mod uart_16550;

pub const UART_ADDRESS: usize = 0x1000_0000;

pub fn shutdown() {
    let address = 0x100000 as *mut u32;
    unsafe { address.write_volatile(0x5555) };
}
