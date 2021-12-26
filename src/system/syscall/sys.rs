#![no_std]
mod api;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}