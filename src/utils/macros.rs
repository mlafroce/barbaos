/// Override de las macros de Rust
#[macro_export]
macro_rules! print
{
    ($($args:tt)+) => ({
        use $crate::devices::Uart;
        use $crate::devices::UART_ADDRESS;
        use core::fmt::Write;
        let _ = write!(Uart::new(UART_ADDRESS), $($args)+);
    });
}

/// Imprime una linea y un salto de linea al final (o una linea vacía)
#[macro_export]
macro_rules! println
{
    () => ({
        print!("\r\n")
    });
    ($fmt:expr) => ({
        print!(concat!($fmt, "\r\n"))
    });
    ($fmt:expr, $($args:tt)+) => ({
        print!(concat!($fmt, "\r\n"), $($args)+)
    });
}
