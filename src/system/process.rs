//! # Procesos
//! Son la base de los sistemas operativos, cada proceso es una instancia
//! de un programa que queremos ejecutar
use crate::assembly::riscv64;
use crate::cpu::riscv64::trap::TrapFrame;
use crate::devices::shutdown;
use crate::init::user_mode_init;
use crate::mmu;
use crate::mmu::map_table::{EntryBits, MapTable};
use crate::mmu::riscv64::{PageTable, PAGE_ORDER, PAGE_SIZE};
use crate::mmu::SIFIVE_TEST_ADDRESS;
use crate::{print, println};
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ptr::NonNull;

/// # Estados del proceso
/// Enumerado con los estados básicos en los que puede estar un proceso.
#[derive(Debug)]
pub enum ProcessState {
    Running,
    Sleeping,
    Waiting,
    Dead,
}

/// Proceso
/// Cada proceso posee los siguientes atributos:
/// * frame: representa el contexto del proceso, es decir, el estado de los
///   registros, stack pointer, mmu, etc.
/// * stack: una porción de memoria para el stack de variables
/// * program_counter
/// * pid: identificador único del proceso
/// * root: tabla de mapeo de memoria
/// * state: estado del proceso
#[repr(C)]
pub struct Process<'a> {
    frame: TrapFrame,
    stack: NonNull<u8>,
    program_counter: usize,
    pid: u16,
    pub root: &'a mut MapTable<'a>,
    state: ProcessState,
    parent_page_table: &'a PageTable,
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

impl<'a> Process<'a> {
    fn new(page_table: &'a PageTable, root: &'a mut MapTable<'a>) -> Self {
        Process {
            frame: TrapFrame::new(),
            stack: page_table.alloc(STACK_PAGES).unwrap(),
            program_counter: 0,
            pid: unsafe { NEXT_PID },
            root,
            state: ProcessState::Waiting,
            parent_page_table: page_table,
        }
    }
    /// Crea un proceso nuevo, que ejecuta la función que le pasamos por
    /// parámetros
    pub fn create(page_table: &'a PageTable) -> Self {
        let root_ptr = page_table.zalloc(1).unwrap().as_ptr() as *mut MaybeUninit<MapTable>;
        let root = unsafe { &mut *root_ptr };
        root.write(MapTable::new(page_table));
        let root_init = unsafe {
            core::mem::transmute::<&mut MaybeUninit<MapTable<'_>>, &mut MapTable<'_>>(root)
        };
        let mut process = Process::new(page_table, root_init);
        unsafe {
            // En un contexto multi-core acá habría una race condition
            NEXT_PID += 1;
        }
        // Mapeo el stack en la MMU
        // Inicializo el stack pointer
        process.frame.regs[SP_REGISTER] = STACK_ADDR + PAGE_SIZE * STACK_PAGES;
        let saddr = process.stack.as_ptr() as usize;
        // Mapeamos la memoria del stack en la memoria virtual del usuario
        // También mapeamos la dirección de la función que va a ejecutar el proceso
        for i in 0..STACK_PAGES {
            let addr = i * PAGE_SIZE;
            process.map_memory(
                STACK_ADDR + addr,
                saddr + addr,
                EntryBits::UserReadWrite.val(),
                0,
            );
        }
        // Actualizo map_page
        process.root.update_satp(process.pid);
        process
    }

    pub fn map_memory(&mut self, vaddr: usize, paddr: usize, bits: i64, level: usize) {
        self.root.map(vaddr, paddr, bits, level);
    }
}

impl Drop for Process<'_> {
    fn drop(&mut self) {
        self.parent_page_table.dealloc(self.stack);
        // el unmap libera a todos los hijos
        self.root.unmap();
        // libera a la raiz
        let ptr = self.root as *mut _ as *mut _;
        unsafe {
            self.parent_page_table.dealloc(NonNull::new_unchecked(ptr));
        }
    }
}

pub struct InitProcess {
    init: UnsafeCell<Option<Process<'static>>>,
}

unsafe impl Sync for InitProcess {}

impl InitProcess {
    fn set_process(process: Process<'static>) {
        let opt = INIT_PROCESS.init.get();
        unsafe { *opt = Some(process) };
    }

    fn get_process() -> &'static Process<'static> {
        let opt = INIT_PROCESS.init.get();
        unsafe { (*opt).as_ref().unwrap() }
    }
}

/// Rust me exige que inicialice todas las variables estáticas
/// Usando un `Option` hago una suerte de lazy init
pub static INIT_PROCESS: InitProcess = InitProcess {
    init: UnsafeCell::new(None),
};

/// Crea el proceso `init`, el proceso que será el padre de todos
/// Llama a `launch_init_process` que a su vez llama a `launch_user_process`
pub fn init(page_table: &'static PageTable) {
    let mut init_process = Process::create(page_table);
    // Temp: mapeo dirección de dispositivo de shutdown.
    init_process.map_memory(
        SIFIVE_TEST_ADDRESS,
        SIFIVE_TEST_ADDRESS,
        EntryBits::UserReadWrite.val(),
        0,
    );
    // Mapeo la dirección de mi proceso en memoria
    // FIX ME?  Mapeo todo el text area porque no conozco las dependencias de mi función init
    let text_start = unsafe { mmu::TEXT_START };
    let text_end = unsafe { mmu::TEXT_END };
    for addr in (text_start..text_end + PAGE_SIZE).step_by(PAGE_SIZE) {
        init_process.map_memory(
            PROCESS_STARTING_ADDR + addr - text_start,
            addr,
            EntryBits::UserReadExecute.val(),
            0,
        );
    }
    let init_root_ref = &mut (*init_process.root);
    unsafe {
        riscv64::mscratch_write(&init_process.frame as *const _ as usize);
        riscv64::satp_fence_asid(init_process.pid as usize);
    }
    // Bueno, quiero que caiga en init_function, pero como no está alineado
    // sale hacer una chanchada
    let func_addr = init_function as *const () as usize;
    let text_start = unsafe { mmu::TEXT_START };
    let text_start_page = (text_start >> PAGE_ORDER) << PAGE_ORDER;
    init_process.program_counter = func_addr - text_start_page + PROCESS_STARTING_ADDR;
    println!("func address: {:x}", func_addr);
    println!(
        "phys address: {:x}",
        init_root_ref
            .virt_to_phys(init_process.program_counter)
            .unwrap()
    );
    InitProcess::set_process(init_process);
    launch_init_process();
}

fn launch_init_process() {
    let init_process = InitProcess::get_process();
    let new_pc = init_process.program_counter;
    let new_sp = init_process.frame.regs[SP_REGISTER];
    unsafe { user_mode_init(new_pc, new_sp) };
}

fn init_function() {
    shutdown();
}