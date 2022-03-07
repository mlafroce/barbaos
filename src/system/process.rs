//! # Procesos
//! Son la base de los sistemas operativos, cada proceso es una instancia
//! de un programa que queremos ejecutar
use crate::assembly::riscv64;
use crate::cpu::riscv64::trap::TrapFrame;
use crate::init::user_mode_init;
use crate::mmu::map_table::{EntryBits, MapTable};
use crate::mmu::riscv64::{PageTable, PAGE_SIZE};
use crate::system::proto::elf_loader::ElfLoader;
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
#[derive(Debug)]
pub struct Process<'a> {
    frame: TrapFrame,
    stack: NonNull<u8>,
    pub program_counter: usize,
    pid: u16,
    pub root: &'a mut MapTable<'a>,
    state: ProcessState,
    parent_page_table: &'a PageTable,
}

/// El registro *sp* es el *x2*
const SP_REGISTER: usize = 2;
/// Por ahora constante, la cantidad de páginas que arman el stack
const STACK_PAGES: usize = 2;
const EXTERNAL_ELF_START: usize = 0x8200_0000;
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
    pub fn set_process(process: Process<'static>) {
        let opt = INIT_PROCESS.init.get();
        unsafe { *opt = Some(process) };
    }

    pub fn get_process() -> &'static Process<'static> {
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
    let init_process = if let Ok(loader) = ElfLoader::new(EXTERNAL_ELF_START as *const u8) {
        loader.into_process(page_table).unwrap()
    } else {
        panic!("ELF not found")
    };
    println!(
        "phys address: {:x}",
        init_process
            .root
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
    unsafe {
        riscv64::mscratch_write(&init_process.frame as *const _ as usize);
        riscv64::satp_fence_asid(init_process.pid as usize);
        user_mode_init(new_pc, new_sp)
    };
}
