use crate::{print, println};
use core::alloc::{AllocError, Allocator, GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr::{null_mut, NonNull};
use core::slice::from_raw_parts_mut;

pub const MTIME_ADDRESS: usize = 0x0200_bff8;
pub const MTIMECMP_ADDRESS: usize = 0x0200_4000;

pub const PAGE_ORDER: usize = 12;

pub const PAGE_SIZE: usize = 1 << PAGE_ORDER;

/// Utilizamos un sistema de alloc en el que dividimos la memoria en N páginas
/// de igual tamaño y luego utilizamos M páginas para guardar información
/// de las mismas
/// N = Tam Heap / Tam Página (usamos páginas de 4096 bytes)
/// M = N / Tam Página
#[derive(Clone, Debug)]
pub struct PageTable {
    heap_start: usize,
    heap_size: usize,
    heap_alloc_start: usize,
}

/// Bits con descripción de las páginas de memoria.
/// Se utilizan 2 de los 8 bits para marcar si la página está libre o no
/// y si es la última en la cadena de allocs.
#[repr(u8)]
pub enum PageBits {
    Empty = 0,
    Used = 0x1,
    Last = 0x2,
}

impl PageBits {
    pub fn val(self) -> u8 {
        self as u8
    }
}

struct Page {
    bits: u8,
}

impl PageTable {
    /// Constructor
    pub const fn new(heap_start: usize, heap_size: usize) -> Self {
        let heap_alloc_start = heap_start;
        Self {
            heap_start,
            heap_size,
            heap_alloc_start,
        }
    }

    /// Inicializamos la tabla de páginas, calculando cuál es la cantidad de
    /// páginas necesaria para cubrir todo el heap.
    ///
    /// Usamos 1 bit para indicar si la página está reservada o no, por lo que reservamos una
    /// página cada PAGE_SIZE páginas
    pub fn init(&mut self) {
        // Cantidad de páginas en la que divido mi heap (incluyendo páginas de estado)
        let num_pages = self.heap_size / PAGE_SIZE;
        // Cantidad de páginas que necesito para guardar información de las páginas
        let reserved_pages = num_pages / (PAGE_SIZE + 1) + 1;
        // Donde comienza la primera página "usable"
        let start_page = self.heap_start as *mut Page;
        for i in 0..num_pages {
            unsafe {
                (*start_page.add(i)).clear();
            }
        }
        self.heap_alloc_start = round_up(self.heap_start + reserved_pages * PAGE_SIZE, PAGE_ORDER);
    }

    /// Reserva N páginas continuas
    pub fn alloc(&self, pages: usize) -> Option<NonNull<u8>> {
        assert!(pages > 0);
        let num_pages = self.heap_size / PAGE_SIZE;
        // Comienzo del heap, inclueyendo páginas que describen el estado de los allocs
        let heap_start_page = self.heap_start as *mut Page;
        let mut free_pages = 0;
        for cur_page_idx in 0..num_pages {
            let cur_page = unsafe { &(*heap_start_page.add(cur_page_idx)) };
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
                unsafe {
                    for offset in first_page..cur_page_idx {
                        (*heap_start_page.add(offset)).set_flag(PageBits::Used);
                    }
                    (*heap_start_page.add(cur_page_idx)).set_flag(PageBits::Used);
                    (*heap_start_page.add(cur_page_idx)).set_flag(PageBits::Last);
                }
                // Devuelvo la página inicial
                // HEAP_ALLOC_START es el heap _luego_ de las páginas reservadas
                // Asumimos que nunca puede ser 0
                let addr = self.heap_alloc_start + PAGE_SIZE * first_page;
                return unsafe { Some(NonNull::new_unchecked(addr as *mut u8)) };
            }
        }
        None
    }

    pub fn zalloc(&self, pages: usize) -> Option<NonNull<u8>> {
        let allocated = self.alloc(pages);
        if let Some(data) = allocated {
            unsafe {
                let pages_slice = from_raw_parts_mut(data.as_ptr(), pages * PAGE_SIZE);
                for item in &mut pages_slice.iter_mut() {
                    *item = 0;
                }
            }
        }
        allocated
    }

    /// Libera páginas reservadas
    pub fn dealloc(&self, ptr: NonNull<u8>) {
        let bits_id = (ptr.as_ptr() as usize - self.heap_alloc_start) / PAGE_SIZE;
        assert!(bits_id < self.heap_size);
        let bits_address = self.heap_start + bits_id;
        unsafe {
            let mut cur_page = bits_address as *mut Page;
            while (*cur_page).is_used() && !(*cur_page).is_last() {
                (*cur_page).clear();
                cur_page = cur_page.add(1);
            }
            // Verificación mínima de double free
            assert!(
                (*cur_page).is_last(),
                "Possible double-free detected! (Not taken found before last)"
            );
            // If we get here, we've taken care of all previous pages and
            // we are on the last page.
            (*cur_page).clear();
        }
    }

    /// Devuelve la cantidad de páginas necesarias para cubrir ese rango de memoria
    pub fn pages_needed(start: usize, end: usize) -> usize {
        (round_up(end, 12) - round_up(start, 12)) / PAGE_SIZE + 1
    }

    /// Devuelve la cantidad de páginas correspondientes al heap
    pub fn get_heap_pages_len(&self) -> usize {
        self.heap_size / PAGE_SIZE
    }

    pub fn print_allocations(&self) {
        unsafe {
            let num_pages = self.heap_size / (PAGE_SIZE + 1);
            let heap_table_beg = self.heap_start as *const Page;
            let heap_table_end = heap_table_beg.add(num_pages);
            let alloc_beg = self.heap_alloc_start;
            let alloc_end = self.heap_alloc_start + num_pages * PAGE_SIZE;
            println!("\x1b[1m[Page Allocation Table]\x1b[0m");
            println!(
                "\x1b[1m\x1b[30mHEAP\x1b[0m: {:p} -> {:p}",
                heap_table_beg, heap_table_end
            );
            println!(
                "\x1b[1m\x1b[30mPHYS\x1b[0m: {:#x} -> {:#x}",
                alloc_beg, alloc_end
            );
            println!("-----------------------");
            let mut total_taken = 0;
            let mut cur_taken = 0;
            let mut cur_start_page = self.heap_alloc_start;
            let mut is_begin = true;
            for i in 0..num_pages {
                if (*heap_table_beg.add(i)).is_used() {
                    if is_begin {
                        cur_start_page = self.heap_alloc_start + PAGE_SIZE * i;
                        is_begin = false;
                    }
                    cur_taken += 1;
                    total_taken += 1;
                    if (*heap_table_beg.add(i)).is_last() {
                        let end_addr = self.heap_alloc_start + PAGE_SIZE * (i + 1) - 1;
                        println!(
                            "Alloc: {:#x} -> {:#x}: {:>3} pages",
                            cur_start_page, end_addr, cur_taken
                        );
                        cur_taken = 0;
                        is_begin = true;
                    }
                }
            }
            println!("-----------------------");
            println!(
                "\x1b[1m\x1b[30mAllocated\x1b[0m: {:>6} pages ({:>10} bytes)",
                total_taken,
                total_taken * PAGE_SIZE
            );
            let total_free = num_pages - total_taken;
            println!(
                "\x1b[1m\x1b[30mFree     \x1b[0m: {:>6} pages ({:>10} bytes)",
                total_free,
                total_free * PAGE_SIZE
            );
        }
    }
}

impl Page {
    pub fn is_used(&self) -> bool {
        self.bits & PageBits::Used.val() != 0
    }

    pub fn is_last(&self) -> bool {
        self.bits & PageBits::Last.val() != 0
    }

    #[allow(dead_code)]
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

#[derive(Copy, Clone)]
pub struct PageAllocator;

unsafe impl GlobalAlloc for PageAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let page_table = GLOBAL_PAGE_TABLE.get_root();
        let pages_needed = (layout.size() >> PAGE_ORDER) + 1;
        if let Some(address) = page_table.alloc(pages_needed) {
            return address.as_ptr();
        }
        null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let page_table = GLOBAL_PAGE_TABLE.get_root();
        if let Some(ptr) = NonNull::new(ptr) {
            page_table.dealloc(ptr);
        }
    }
}

unsafe impl Allocator for PageAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let page_table = GLOBAL_PAGE_TABLE.get_root();
        let pages_needed = (layout.size() >> PAGE_ORDER) + 1;
        if let Some(ptr) = page_table.alloc(pages_needed) {
            let fat_ptr;
            unsafe {
                let array = from_raw_parts_mut(ptr.as_ptr(), layout.size());
                fat_ptr = NonNull::new_unchecked(array as *mut [u8]);
            }
            Ok(fat_ptr)
        } else {
            Err(AllocError)
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, _layout: Layout) {
        let page_table = GLOBAL_PAGE_TABLE.get_root();
        page_table.dealloc(ptr);
    }
}

pub struct GlobalPageTable {
    root: UnsafeCell<PageTable>,
}

impl GlobalPageTable {
    const fn empty() -> Self {
        let root = UnsafeCell::new(PageTable::new(0, 0));
        Self { root }
    }

    pub unsafe fn set_root(&self, root: &PageTable) {
        *self.root.get() = root.clone();
    }

    pub fn get_root(&self) -> &PageTable {
        unsafe { &*self.root.get() }
    }
}

unsafe impl Sync for GlobalPageTable {}

#[global_allocator]
pub static PAGE_ALLOCATOR: PageAllocator = PageAllocator {};
pub static GLOBAL_PAGE_TABLE: GlobalPageTable = GlobalPageTable::empty();
