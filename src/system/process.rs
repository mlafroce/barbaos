//! # Procesos
//! Son la base de los sistemas operativos, cada proceso es una instancia
//! de un programa que queremos ejecutar
use crate::{print, println};
use crate::devices::uart::Uart;
use crate::cpu;
use crate::cpu::trap::TrapFrame;
use crate::mmu::map_table::{EntryBits, MapTable};
use crate::mmu::page_table::{PAGE_SIZE, PageTable};

/// # Estados del proceso
/// Enumerado con los estados básicos en los que puede estar un proceso.
pub enum ProcessState {
    Running,
    Sleeping,
    Waiting,
    Dead,
}

/// Proceso
/// Cada proceso posee los siguientes atributos:
/// * frame: representa el contexto del proceso, es decir, el estado de los
/// registros, stack pointer, mmu, etc.
/// * stack: una porción de memoria para el stack de variables
/// * program_counter
/// * pid: identificador único del proceso
/// * root: tabla de mapeo de memoria
/// * state: estado del proceso
#[repr(C)]
pub struct Process {
    frame:           TrapFrame,
    stack:           *mut u8,
    program_counter: usize,
    pid:             u16,
    root:            *mut MapTable,
    state:           ProcessState,
}

/// El registro *sp* es el *x2*
const SP_REGISTER: usize = 2;
/// Por ahora constante, la cantidad de páginas que arman el stack
const STACK_PAGES: usize = 2;
/// Técnicamente mi proceso puede estar ubicado en cualquier lugar
/// ¡es memoria virtual!
const PROCESS_STARTING_ADDR: usize = 0x2000_0000;
/// Dónde arranca el stack (recordar que va de arriba hacia abajo)
pub const STACK_ADDR: usize = 0x1_0000_0000;

/// PID global: cuando sea multithread, proteger
static mut NEXT_PID: u16 = 0;

impl Process {
    /// Crea un proceso nuevo, que ejecuta la función que le pasamos por
    /// parámetros
    pub fn new_default(func: fn()) -> Self {
    let func_addr = func as usize;
    
    let mut process = Process {
            frame:           TrapFrame::new(),
            stack:           PageTable::alloc(STACK_PAGES).unwrap(),
            program_counter: PROCESS_STARTING_ADDR,
            pid:             unsafe { NEXT_PID },
            root:            PageTable::zalloc(1).unwrap() as *mut MapTable,
            state:           ProcessState::Waiting,
        };
    unsafe {
        // En un contexto multi-core acá habría una race condition
        NEXT_PID += 1;
    }
    // Inicializo el stack pointer
    process.frame.regs[SP_REGISTER] = STACK_ADDR + PAGE_SIZE * STACK_PAGES;
    // Mapeo el stack en la MMU
    let root_table;
    unsafe {
      root_table = &mut *process.root;
    }
    let saddr = process.stack as usize;
    // Mapeamos la memoria del stack en la memoria virtual del usuario
    // También mapeamos la dirección de la función que va a ejecutar el proceso
    for i in 0..STACK_PAGES {
      let addr = i * PAGE_SIZE;
      root_table.map(
          STACK_ADDR + addr,
          saddr + addr,
          EntryBits::UserReadWrite.val(),
          0,
      );
    }
    // Map the program counter on the MMU
    root_table.map(
        PROCESS_STARTING_ADDR,
        func_addr,
        EntryBits::UserReadExecute.val(),
        0,
    );
    root_table.update_satp(process.pid);
    process
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        PageTable::dealloc(self.stack);
        unsafe {
            // el unmap libera a todos los hijos
            let root_ref: &mut MapTable = &mut (*self.root);
            root_ref.unmap();
        }
        // libera a la raiz
        PageTable::dealloc(self.root as *mut u8);
    }
}

/// Rust me exige que inicialice todas las variables estáticas
/// Usando un `Option` hago una suerte de lazy init
static mut INIT_PROCESS: Option<Process> = None;

/// Crea el proceso `init`, el proceso que será el padre de todos
/// Devuelve el valor del Program counter del proceso
pub fn init() -> usize {
    let init_process = Process::new_default(init_function);
    let init_root_ref: &mut MapTable;
    unsafe {
        init_root_ref = &mut (*init_process.root);
        cpu::mscratch_write(&init_process.frame as *const _ as usize);
        INIT_PROCESS = Some(init_process);
    }
    // Bueno, quiero que caiga en init_function, pero como no está alineado
    // sale hacer una chanchada
    let func_addr = init_function as *const () as usize;
    let func_virt_addr = PROCESS_STARTING_ADDR + func_addr % PAGE_SIZE;
    println!("func address: {:x}", func_addr);
    println!("phys address: {:x}", init_root_ref.virt_to_phys(func_virt_addr).unwrap());
    func_virt_addr
}

fn init_function () {
    loop {}
}
