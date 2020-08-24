use crate::{print, println};
use core::fmt::{Error, Write};

/// Dispositivo UART
/// Escribimos un "driver" de un dispositivo NS16650a
/// Como esto es emulado en QEMU, nos tomamos algunas libertades
/// base_addres es la dirección del dispositivo mapeada a memoria
pub struct Uart {
    base_address: usize,
}

impl Uart {
    pub fn new(base_address: usize) -> Self {
        Uart { base_address }
    }

    /// Inicializamos el "driver"
    /// https://en.wikipedia.org/wiki/16550_UART
    /// @param base_addr es la dirección de memoria a la que está mapeado este dispositivo
    /// La configuración se puede hacer siguiendo la descripción de su datasheet
    /// https://osblog.stephenmarz.com/ch2.html
    pub fn init(&self) {
        let ptr = self.base_address as *mut u8;
        unsafe {
            // Longitud de word transmitido: 8 bits
            let lcr = 0b00000011;
            ptr.add(3).write_volatile(lcr);

            // Habilita FIFO de caracteres en el dispositivo
            ptr.add(2).write_volatile(0b00000001);

            // Habilita interrupciones, pero no las vamos a manejar aun
            ptr.add(1).write_volatile(0b00000001);

            // Acá podríamos ajustar el divisor de comunicación del dispositivo
            // Pero como es emulado, saltamos toda esa parte
        }
    }

    #[allow(dead_code)]
    pub fn get_char(&self) -> Option<u8> {
        let ptr = self.base_address as *mut u8;
        unsafe {
            // El bit 5 es el Line Control Register, y nos indica si leyó algo o no.
            if ptr.add(5).read_volatile() & 1 == 0 {
                // No hay datos
                None
            } else {
                // Hay datos! Lo levanto
                Some(ptr.add(0).read_volatile())
            }
        }
    }

    fn put_char(&self, c: u8) {
        let ptr = self.base_address as *mut u8;
        unsafe {
            // Asumimos que el transmisor está vacío
            ptr.add(0).write_volatile(c);
        }
    }
}

pub fn read_uart(uart: &Uart) {
    if let Some(c) = uart.get_char() {
        match c {
            8 | 127 => {
                // Backspace
                print!("{} {}", 8 as char, 8 as char);
            }
            10 | 13 => {
                // Newline or carriage-return
                println!();
            }
            _ => {
                print!("{}", c as char);
            }
        }
    }
}

/// Utilizamos el dispositivo como canal de escritura
impl Write for Uart {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for c in s.bytes() {
            self.put_char(c);
        }
        Ok(())
    }
}
