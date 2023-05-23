use crate::{kmain, print, println, DTB_ADDRESS};
use core::arch::{asm, global_asm};
use core::ptr::write_volatile;

global_asm!(include_str!("../asm/armv7a/boot.S"));
global_asm!(include_str!("../asm/armv7a/trap.S"));
global_asm!(include_str!("../asm/armv7a/mem.S"));

#[inline]
pub unsafe fn wfi() {
    asm!("wfi", options(nomem, nostack))
}

#[inline]
pub unsafe fn isb() {
    asm!("isb", options(nomem, nostack))
}

#[repr(C)]
struct IrqController {
    irq_basic_pending: u32,
    irq_pending_1: u32,
    irq_pending_2: u32,
    fiq_control: u32,
    enable_irqs_1: u32,
    enable_irqs_2: u32,
    enable_basic_irqs: u32,
    disable_irqs_1: u32,
    disable_irqs_2: u32,
    disable_basic_irqs: u32,
}

#[repr(C)]
#[allow(unused)]
struct TimerCtrl {
    load: u32,
    value: u32,
    control: u32,
    irq_clear: u32,
    raw_irq: u32,
    masked_irq: u32,
    reload: u32,
    pre_divider: u32,
    free_running_counter: u32,
}

#[allow(unused)]
#[repr(C)]
struct SysTimerRegister {
    status: u32,
    counter_low: u32,
    counter_hi: u32,
    compare_0: u32,
    compare_1: u32,
    compare_2: u32,
    compare_3: u32,
}

#[repr(C)]
#[derive(Debug)]
struct GicRegisters {
    icr: u32,
    pmr: u32,
    bpr: u32,
    iar: u32,
    eoir: u32,
    rpr: u32,
}

#[inline]
pub unsafe fn enable_interrupts() {
    asm!("mrs {status}, cpsr",
    "bic {temp}, {status}, #0x80",
    "eor {temp}, {temp}, #0",
    /* Enable IRQs */
    "msr cpsr_c, {temp}",
    status = out(reg) _,
    temp = out(reg) _);
}

#[inline]
pub unsafe fn irq_init() {
    // Irq init
    let timer_irq: *mut IrqController = 0x3F00_B200 as *mut _;
    write_volatile(&mut (*timer_irq).disable_basic_irqs, u32::MAX);
    write_volatile(&mut (*timer_irq).disable_irqs_1, u32::MAX);
    write_volatile(&mut (*timer_irq).disable_irqs_2, u32::MAX);
    write_volatile(&mut (*timer_irq).enable_irqs_1, 0b101);
    // Local irq init
    let timer_ctrl: *mut u32 = 0x4000_0040 as *mut _;
    write_volatile(timer_ctrl, 0);
    let mbox_ctrl: *mut u32 = 0x4000_0050 as *mut _;
    write_volatile(mbox_ctrl, 0);
}

#[inline]
pub unsafe fn init_timer() {
    let clock = 19_200_000;
    // https://developer.arm.com/documentation/ddi0500/e/system-control/aarch32-register-summary/c14-registers
    // c14, c0, 0:  CNTFRQ
    asm!("mcr p15, 0, {}, c14, c0, 0", in (reg) clock, options(nostack));
    // c14, c2, 0:  CNTP_TVAL
    asm!("mcr p15, 0, {}, c14, c2, 0", in (reg) clock, options(nostack));
    // Enable CNTP
    let flag = 1;
    asm!("mcr p15, 0, {}, c14, c2, 1", in (reg) flag, options(nostack));
    isb();
    // Interrupts
    let timer_ctrl: *mut u32 = 0x4000_0040 as *mut _;
    write_volatile(timer_ctrl, 1);
}

#[inline]
pub unsafe fn queue_timer() {
    let clock = 19_200 * 1000;
    let cur_val: u32;
    // c14, c2, 0:  CNTP_TVAL
    asm!("mrc p15, 0, {}, c14, c2", out (reg) cur_val);
    println!("Cur val {:x}: ", cur_val);
    println!("Sum {:x}: ", clock);
    let new_val = cur_val + clock;
    // set value for irq
    asm!("mcr p15, 0, {}, c14, c2, 0", in (reg) new_val);
    // Enable CNTP
    let flag = 1;
    asm!("mcr p15, 0, {}, c14, c2, 1", in (reg) flag, options(nomem, nostack));
}

#[no_mangle]
extern "C" fn machine_init(_zero: usize, _hart_id: usize, dtb_address: *const u8) {
    // Cargo dirección de memoria de kinit
    unsafe {
        DTB_ADDRESS = dtb_address;
    }
    kmain();
}

#[no_mangle]
pub unsafe extern "C" fn handle_irq() {
    println!("Irq handled");
    //let gic_iface: *mut GicRegisters = 0x3e00_b200 as *mut _;
    //let eoir = &(*gic_iface);
    //let reg = unsafe { read_volatile(gic_iface) };
    //unsafe { (*gic_iface).eoir = 0x};
    //unsafe { println!("IRQ Handled {:?}", eoir) };
}

#[no_mangle]
pub unsafe extern "C" fn handle_unsupported() {
    println!("Unsupported exception");
    asm!("wfi");
}

#[no_mangle]
pub unsafe extern "C" fn handle_prefetch_abort() {
    println!("Prefetch aborted (memory fault)");
    asm!("wfi");
}
