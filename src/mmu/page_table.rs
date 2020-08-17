use crate::{print, println};
use crate::devices::uart::Uart;

use core::slice::from_raw_parts_mut;


extern "C" {
    pub static HEAP_START: usize;
    pub static HEAP_SIZE: usize;
}

static mut HEAP_ALLOC_START: usize = 0;
const PAGE_ORDER: usize = 12;

pub const PAGE_SIZE: usize = 1 << PAGE_ORDER;

/// Utilizamos un sistema de alloc en el que dividimos la memoria en N páginas
/// de igual tamaño y luego utilizamos M páginas para guardar información
/// de las mismas
/// N = Tam Heap / Tam Página (usamos páginas de 4096 bytes)
/// M = N / Tam Página
pub struct PageTable {}

/// Bits con descripción de las páginas de memoria.
/// Se utilizan 2 de los 8 bits para marcar si la página está libre o no
/// y si es la última en la cadena de allocs.
#[repr(u8)]
pub enum PageBits {
    Empty = 0,
    Used = 0x1,
    Last = 0x2
}

impl PageBits {
    pub fn val(self) -> u8 {
        self as u8
    }
}

struct Page {
    bits: u8
}

impl PageTable {
    /// Inicializamos la tabla de páginas, calculando cuál es la cantidad de 
    /// páginas necesaria para cubrir todo el heap. Una vez que sabemos la cantidad
    /// de páginas (*N*), reservamos *M páginas para guardar N bytes con la información
    /// de paginado.
    pub fn init() {
        unsafe {
            // Cantidad de páginas en la que divido mi heap
            let num_pages = HEAP_SIZE / PAGE_SIZE;
            // Cantidad de páginas que necesito para guardar información de las páginas
            let reserved_pages = num_pages / PAGE_SIZE;
            // Donde comienza la primer página "usable"
            let start_page = HEAP_START as *mut Page;
            for i in 0..num_pages {
                (*start_page.add(i)).clear();
            }
            HEAP_ALLOC_START = round_up(HEAP_START + reserved_pages * PAGE_SIZE, PAGE_ORDER);
        }
    }

    /// Reserva N páginas continuas
    pub fn alloc(pages: usize) -> Option<*mut u8> {
        assert!(pages > 0);
        unsafe {
            let num_pages = HEAP_SIZE / PAGE_SIZE;
            // Comienzo del heap, inclueyendo páginas que describen el estado de los allocs
            let heap_start_page = HEAP_START as *mut Page;
            let mut free_pages = 0;
            for cur_page_idx in 0..num_pages - pages {
                let cur_page = &(*heap_start_page.add(cur_page_idx));
                if !cur_page.is_used() {
                    // Si está libre, lo cuento
                    free_pages += 1;
                } else {
                    // Si no está libre reseteo la cantidad de páginas encontradas
                    free_pages = 0;
                }
                // Encontré N paginas libres consecutivas?
                if free_pages == pages {
                    let first_page = cur_page_idx + 1 - pages;
                    // Voy reservando memoria
                    for offset in first_page..cur_page_idx {
                        (*heap_start_page.add(offset)).set_flag(PageBits::Used);
                    }
                    (*heap_start_page.add(cur_page_idx)).set_flag(PageBits::Used);
                    (*heap_start_page.add(cur_page_idx)).set_flag(PageBits::Last);
                    // Devuelvo la página inicial
                    // HEAP_ALLOC_START es el heap _luego_ de las páginas reservadas
                    return Some((HEAP_ALLOC_START + PAGE_SIZE * first_page) as *mut u8);
                }
            }
            None
        }
    }

    #[allow(dead_code)]
    pub fn zalloc(pages: usize) -> Option<*mut u8> {
        let allocated = PageTable::alloc(pages);
        match allocated {
            Some(page) => {
                unsafe {
                    let pages_slice = from_raw_parts_mut(page, pages * PAGE_SIZE);
                    for item in &mut pages_slice.iter_mut() {
                        *item = 0;
                    }
                }
                Some(page)
            }
            None => None,
        }
    }

    /// Libera páginas reservadas
    pub fn dealloc(ptr: *mut u8) {
        if !ptr.is_null() {
            unsafe {
                let bits_id = (ptr as usize - HEAP_ALLOC_START) / PAGE_SIZE;
                assert!(bits_id < HEAP_SIZE);
                let bits_address = HEAP_START + bits_id;
                let mut cur_page = bits_address as *mut Page;
                while (*cur_page).is_used() && !(*cur_page).is_last() {
                    (*cur_page).clear();
                    cur_page = cur_page.add(1);
                }
                // Verificación mínima de double free
                assert!( (*cur_page).is_last(),
                    "Possible double-free detected! (Not taken found before last)"
                );
                // If we get here, we've taken care of all previous pages and
                // we are on the last page.
                (*cur_page).clear();
            }
        }
    }

    /// Devuelve la cantidad de páginas necesarias para cubrir ese rango de memoria
    pub fn pages_needed(start: usize, end: usize) -> usize {
        (round_up(end, 12) - round_up(start, 12)) / PAGE_SIZE + 1
    }

    /// Devuelve la cantidad de páginas correspondientes al heap
    ///
    /// # Safety
    /// No tiene problemas de seguridad, las variables se inicializan por el linker
    pub unsafe fn get_heap_pages_len() -> usize {
        HEAP_SIZE / PAGE_SIZE
    }

    pub fn print_allocations() {
        unsafe {
            let num_pages = HEAP_SIZE / PAGE_SIZE;
            let heap_beg = HEAP_START as *const Page;
            let heap_end = heap_beg.add(num_pages);
            let alloc_beg = HEAP_ALLOC_START;
            let alloc_end = HEAP_ALLOC_START + num_pages * PAGE_SIZE;
            println!("\x1b[1m[Page Allocation Table]\x1b[0m");
            println!("\x1b[1m\x1b[30mHEAP\x1b[0m: {:p} -> {:p}", heap_beg, heap_end);
            println!("\x1b[1m\x1b[30mPHYS\x1b[0m: {:#x} -> {:#x}", alloc_beg, alloc_end);
            println!("-----------------------");
            let mut total_taken = 0;
            let mut cur_taken = 0;
            let mut cur_start_page = HEAP_ALLOC_START;
            let mut is_begin = true;
            for i in 0..num_pages {
                if (*heap_beg.add(i)).is_used() {
                    if is_begin {
                        cur_start_page = HEAP_ALLOC_START + PAGE_SIZE * i;
                        is_begin = false;
                    }
                    cur_taken += 1;
                    total_taken += 1;
                    if (*heap_beg.add(i)).is_last() {
                        let end_addr = HEAP_ALLOC_START + PAGE_SIZE * (i + 1) - 1;
                        println!("Alloc: {:#x} -> {:#x}: {:>3} pages", cur_start_page, end_addr, cur_taken);
                        cur_taken = 0;
                        is_begin = true;
                    }
                }
            }
            println!("-----------------------");
            println!("\x1b[1m\x1b[30mAllocated\x1b[0m: {:>6} pages ({:>10} bytes)",
                total_taken, total_taken * PAGE_SIZE);
            let total_free = num_pages - total_taken;
            println!("\x1b[1m\x1b[30mFree     \x1b[0m: {:>6} pages ({:>10} bytes)",
                total_free, total_free * PAGE_SIZE);
        }
    }
}

impl Page {
    pub fn is_used(&self) -> bool {
        self.bits & PageBits::Used.val() != 0
    }

    pub fn is_last(& self) -> bool {
        self.bits & PageBits::Last.val() != 0
    }

    pub fn set_flag(&mut self, flags: PageBits) {
        self.bits |= flags.val();
    }

    pub fn clear(&mut self) {
        self.bits = PageBits::Empty.val();
    }
}

fn round_up(val: usize, order: usize) -> usize {
    let mask = (1usize << order) - 1;
    (val + mask) & !mask
}
