use core::arch::asm;
use core::fmt::{Error, Write};

const MMIO_BASE: usize = 0x3F000000;
const GPIO_BASE: usize = MMIO_BASE + 0x200000;
const GPFSEL1: usize = GPIO_BASE + 0x4;
const GPPUD: usize = GPIO_BASE + 0x94;
// Controls actuation of pull up/down for specific GPIO pin.
const GPPUDCLK0: usize = GPIO_BASE + 0x98;

// The base address for UART.
const UART0_BASE: usize = GPIO_BASE + 0x1000;
// for raspi4 0xFE201000, raspi2 & 3 0x3F201000, and 0x20201000 for raspi1

// The offsets for reach register for the UART.
const UART0_DR: usize = 0x00;
const UART0_RSRECR: usize = 0x04;
const UART0_FR: usize = 0x18;
const UART0_ILPR: usize = 0x20;
const UART0_IBRD: usize = 0x24;
const UART0_FBRD: usize = 0x28;
const UART0_LCRH: usize = 0x2C;
const UART0_CR: usize = 0x30;
const UART0_IFLS: usize = 0x34;
const UART0_IMSC: usize = 0x38;
const UART0_RIS: usize = 0x3C;
const UART0_MIS: usize = 0x40;
const UART0_ICR: usize = 0x44;
const UART0_DMACR: usize = 0x48;
const UART0_ITCR: usize = 0x80;
const UART0_ITIP: usize = 0x84;
const UART0_ITOP: usize = 0x88;
const UART0_TDR: usize = 0x8C;

const AUX_ENABLES: usize = 0x3F215004;
const AUX_MU_IO_REG: usize = 0x3F215040;
const AUX_MU_IER_REG: usize = 0x3F215044;
const AUX_MU_IIR_REG: usize = 0x3F215048;
const AUX_MU_LCR_REG: usize = 0x3F21504C;
const AUX_MU_MCR_REG: usize = 0x3F215050;
const AUX_MU_LSR_REG: usize = 0x3F215054;
const AUX_MU_MSR_REG: usize = 0x3F215058;
const AUX_MU_SCRATCH: usize = 0x3F21505C;
const AUX_MU_CNTL_REG: usize = 0x3F215060;
const AUX_MU_STAT_REG: usize = 0x3F215064;
const AUX_MU_BAUD_REG: usize = 0x3F215068;

/// Dispositivo UART
/// Escribimos un "driver" de un dispositivo MINI-Uart
/// Este driver es más estricto ya que es hardware real
/// base_addres es la dirección del dispositivo mapeada a memoria
pub struct Uart {
    base_address: usize,
}

#[inline]
pub unsafe fn delay(cycles: usize) {
    asm!(
    // Use local labels to avoid R_ARM_THM_JUMP8 relocations which fail on thumbv6m.
    "1:",
    "subs {}, #1",
    "bne 1b",
    inout(reg) cycles => _,
    options(nomem, nostack),
    )
}

fn put_32(address: usize, data: u32) {
    let ptr = address as *mut u32;
    unsafe { ptr.write_volatile(data) };
}

fn get_32(address: usize) -> u32 {
    let ptr = address as *mut u32;
    unsafe { ptr.read_volatile() }
}

impl Uart {
    pub fn new(base_address: usize) -> Self {
        Uart { base_address }
    }

    /// Inicializamos el "driver"
    /// @param base_addr es la dirección de memoria a la que está mapeado este dispositivo
    /// La configuración se puede hacer siguiendo la descripción de su datasheet
    pub fn init(&self) {
        let ptr = self.base_address as *mut u32;
        unsafe {
            ptr.add(UART0_CR / 4).write_volatile(0);

            /*
            let mut ra = get_32(GPFSEL1);
            ra &= !(7<<12); //gpio14
            ra |= 4<<12;    //alt0
            ra &= !(7<<15); //gpio14
            ra |= 4<<15;    //alt0
            put_32(GPFSEL1,ra);
            */

            (GPPUD as *mut u32).write_volatile(0);
            delay(150);
            // Disable pull up/down for pin 14,15 & delay for 150 cycles.
            (GPPUDCLK0 as *mut u32).write_volatile((1 << 14) | (1 << 15));
            delay(150);
            // Write 0 to GPPUDCLK0 to make it take effect.
            (GPPUDCLK0 as *mut u32).write_volatile(0);

            // Clear pending interrupts.
            ptr.add(UART0_ICR / 4).write_volatile(0x7FF);
            // Divider = 3000000 / (16 * 115200) = 1.627 = ~1.
            ptr.add(UART0_IBRD / 4).write_volatile(1);
            // Fractional part register = (.627 * 64) + 0.5 = 40.6 = ~40.
            ptr.add(UART0_FBRD / 4).write_volatile(40);

            // Enable FIFO & 8 bit data transmission (1 stop bit, no parity).
            ptr.add(UART0_LCRH / 4)
                .write_volatile((1 << 4) | (1 << 5) | (1 << 6));
            ptr.add(UART0_IMSC / 4).write_volatile(
                (1 << 1)
                    | (1 << 4)
                    | (1 << 5)
                    | (1 << 6)
                    | (1 << 7)
                    | (1 << 8)
                    | (1 << 9)
                    | (1 << 10),
            );

            // Enable UART0, receive & transfer part of UART.
            ptr.add(UART0_CR / 4)
                .write_volatile((1 << 0) | (1 << 8) | (1 << 9));
        }
    }

    pub fn init2(&self) {
        unsafe {
            put_32(AUX_ENABLES, 1);
            put_32(AUX_MU_IER_REG, 0);
            put_32(AUX_MU_CNTL_REG, 0);
            put_32(AUX_MU_LCR_REG, 3);
            put_32(AUX_MU_MCR_REG, 0);
            put_32(AUX_MU_IER_REG, 0);
            put_32(AUX_MU_IIR_REG, 0xC6);
            put_32(AUX_MU_BAUD_REG, 270);
            let mut ra = get_32(GPFSEL1);
            ra &= !(7 << 12); //gpio14
            ra |= 2 << 12; //alt5
            put_32(GPFSEL1, ra);

            put_32(GPPUD, 0);
            delay(150);
            put_32(GPPUDCLK0, 1 << 14);
            delay(150);
            put_32(GPPUDCLK0, 0);

            put_32(AUX_MU_CNTL_REG, 2);
        }
    }

    #[allow(dead_code)]
    fn get_char(&self) -> Option<u8> {
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
            while (ptr.add(UART0_FR).read_volatile() & 0x20) != 0 {
                delay(1);
            }
            ptr.add(UART0_DR).write_volatile(c);
            //put_32(AUX_MU_IO_REG, c as u32)
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
