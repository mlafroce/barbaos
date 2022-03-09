#![no_std]
mod api;
mod newlib_api;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}