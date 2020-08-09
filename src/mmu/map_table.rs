use super::riscv64::{PageTable, PAGE_SIZE};
use crate::cpu::riscv64::plic;
use crate::{print, println};

use core::ptr::NonNull;
use core::slice::from_raw_parts_mut;

const SATP_MODE_SV39: usize = 8 << 60;

#[repr(i64)]
#[derive(Copy, Clone)]
#[allow(dead_code)]
pub enum EntryBits {
    None = 0,
    Valid = 1 << 0,
    Read = 1 << 1,
    Write = 1 << 2,
    Execute = 1 << 3,
    User = 1 << 4,
    Global = 1 << 5,
    Access = 1 << 6,
    Dirty = 1 << 7,

    ReadWrite = 1 << 1 | 1 << 2,
    ReadExecute = 1 << 1 | 1 << 3,
    ReadWriteExecute = 1 << 1 | 1 << 2 | 1 << 3,
}

impl EntryBits {
    pub fn val(self) -> i64 {
        self as i64
    }
}

/// Entrada en la tabla de paginación
#[derive(Copy, Clone)]
pub struct Entry {
    pub entry: i64,
}

impl Entry {
    pub fn new() -> Self {
        Entry { entry: 0 }
    }

    pub fn is_valid(&self) -> bool {
        self.get_entry() & EntryBits::Valid.val() != 0
    }

    // Es hoja si tiene algún flag RXE
    pub fn is_leaf(&self) -> bool {
        self.get_entry()
            & (EntryBits::Read.val() | EntryBits::Write.val() | EntryBits::Execute.val())
            != 0
    }

    pub fn set_entry(&mut self, entry: i64) {
        self.entry = entry;
    }

    pub fn get_entry(&self) -> i64 {
        self.entry
    }
}

impl Default for Entry {
    fn default() -> Self {
        Entry::new()
    }
}

/// Tabla con información para mapear memoria virtual a memoria física
#[repr(C)]
pub struct MapTable<'a> {
    pub entries: [Entry; 512],
    page_table: &'a PageTable,
}

impl<'a> MapTable<'a> {
    pub fn new(page_table: &'a PageTable) -> Self {
        MapTable {
            page_table,
            entries: [Entry::new(); 512],
        }
    }
    /// Relaciona, en la tabla, una dirección de memoria virtual con dirección física.
    ///
    /// # Arguments
    ///
    /// * `vaddr` - Dirección virtual
    /// * `paddr` - Dirección física a la que se mapea
    /// * `bits`  - Bits de descripción de la entrada
    /// * `level` - Nivel de la página: 2 para página de 1GB, 1 para 2MB y 0 para 4KB
    pub fn map(&mut self, vaddr: usize, paddr: usize, bits: i64, level: usize) {
        // Si los bits no corresponden a una hoja, voy a tener leaks y pagefaults
        assert!(bits & 0xe != 0);
        // Desarmo la dirección virtual
        let vpn = [
            // VPN[0] = vaddr[20:12]
            (vaddr >> 12) & 0x1ff,
            // VPN[1] = vaddr[29:21]
            (vaddr >> 21) & 0x1ff,
            // VPN[2] = vaddr[38:30]
            (vaddr >> 30) & 0x1ff,
        ];
        let mut cur_table = &mut self.entries[vpn[2]];
        for i in (level..2).rev() {
            if !cur_table.is_valid() {
                // La entrada no está reservada, la reservo
                let new_page = self.page_table.zalloc(1).unwrap();
                // Reservo la entrada
                // La dirección tiene 12 ceros del lado menos significativo
                // En las entradas de la TBL la dirección va del bit 10 al 53
                let entry_address = new_page.as_ptr() as i64 >> 2;
                cur_table.set_entry(entry_address | EntryBits::Valid.val());
            }
            // Restauro offset de páginas. Entry no solo es una entrada en la TBL, si esta TBL es rama
            // es también la dirección a una TBL potencialmente hoja
            let entry = ((cur_table.get_entry() & !0x3ff) << 2) as *mut Entry;
            // cur_table ahora es la tabla en la dirección de memoria de la tabla anterior[vpn[i]]
            // Ej: arranco con mi tabla raiz, cur_table era una tabla en la entrada vpn[2]. Ahora cur_tabla será
            // esta tabla y buscaré la entrada correspondiente a VPN[1]
            let next_table = unsafe { entry.add(vpn[i]).as_mut() };
            cur_table = next_table.unwrap();
        }
        // cur_table ahora es una entrada en una hoja. Ajusto la dirección física a la entrada
        let phys_entry_addr = (paddr >> 12) << 10;
        let entry = phys_entry_addr as i64 | bits | EntryBits::Valid.val() | EntryBits::Dirty.val() |  // Some machines require this to =1
                EntryBits::Access.val();
        // Guardo la entrada en la tabla de nivel `level`
        cur_table.set_entry(entry);
    }

    /// Elimina mapeos creados con la función `map`.
    #[allow(dead_code)]
    pub fn unmap(&mut self) {
        MapTable::unmap_entries(self.page_table, &mut self.entries);
    }

    /// Version recursiva
    #[allow(dead_code)]
    fn unmap_entries(page_table: &PageTable, entries: &mut [Entry]) {
        for cur_entry in entries {
            if cur_entry.is_valid() && !cur_entry.is_leaf() {
                // Reconstruyo dirección virtual
                let child_table_addr = (cur_entry.get_entry() & !0x3ff) << 2;
                let child_entries =
                    unsafe { from_raw_parts_mut(child_table_addr as *mut Entry, 512) };
                // Libero los hijos de la tabla hija
                MapTable::unmap_entries(page_table, child_entries);
                unsafe {
                    page_table.dealloc(NonNull::new_unchecked(child_table_addr as *mut u8));
                }
            }
            cur_entry.set_entry(0);
        }
    }

    /// Convierte una dirección virtual en una física.
    pub fn virt_to_phys(&self, vaddr: usize) -> Option<usize> {
        // Desarmo la dirección virtual
        let vpn = [
            // VPN[0] = vaddr[20:12]
            (vaddr >> 12) & 0x1ff,
            // VPN[1] = vaddr[29:21]
            (vaddr >> 21) & 0x1ff,
            // VPN[2] = vaddr[38:30]
            (vaddr >> 30) & 0x1ff,
        ];
        let mut cur_table = &self.entries[vpn[2]];
        for i in (0..=2).rev() {
            if !cur_table.is_valid() {
                // Dirección virtual inválida, page fault.
                break;
            }
            if cur_table.is_leaf() {
                // Depende el nivel voy a tener un tamaño de offset
                // Por ejemplo, el nivel 0 tiene 12 bits de offset relativos
                // a la página.
                // En el nivel 1 las páginas tienen 21 bits de offset, etc
                let offset_mask = (1 << (12 + i * 9)) - 1;
                let page_offset = vaddr & offset_mask;
                let phys_addr = ((cur_table.get_entry() << 2) as usize) & !offset_mask;
                println!(
                    "Entry_table {:x}: {:x}",
                    (cur_table.entry << 2) as usize & !offset_mask,
                    (cur_table.entry & EntryBits::Execute.val())
                );
                return Some(phys_addr | page_offset);
            } else {
                // Si no es hoja, ingresamos a la rama como en `map`
                let entry = ((cur_table.get_entry() & !0x3ff) << 2) as *mut Entry;
                let entry_child = vpn[i - 1];
                // cur_table ahora es la tabla en la dirección de memoria de la tabla anterior[vpn[i]]
                cur_table = unsafe { entry.add(entry_child).as_mut().unwrap() };
            }
        }
        None
    }

    /// Mapea direcciones virtuales a una física del mismo valor
    /// Utiliza páginas de 4KB
    pub fn range_map(&mut self, start: usize, end: usize, bits: i64) {
        let mut phys_addr = start & !(PAGE_SIZE - 1);
        let num_pages = PageTable::pages_needed(start, end);
        for _ in 0..num_pages {
            self.map(phys_addr, phys_addr, bits, 0);
            phys_addr += 1 << 12;
        }
    }

    /// Mapea direcciones virtuales a las físicas inicializadas en modo máquina
    ///
    /// # Safety
    /// No tiene problemas de seguridad, las variables se inicializan por el linker
    pub unsafe fn init_map(&mut self) {
        let entries_address = &self.entries as *const _ as usize;
        self.map(
            entries_address,
            entries_address,
            EntryBits::ReadWrite.val(),
            0,
        );
        // RODATA comparte espacio en la página con TEXT, así que primero mapeo
        // RODATA como sólo lectura y después mapeo TEXT como ejecutable
        // De esta forma me aseguro que la última página sea ejecutable
        self.range_map(
            super::RODATA_START,
            super::RODATA_END,
            EntryBits::Read.val(),
        );
        self.range_map(
            super::TEXT_START,
            super::TEXT_END,
            EntryBits::ReadExecute.val(),
        );
        self.range_map(
            super::DATA_START,
            super::DATA_END,
            EntryBits::ReadWrite.val(),
        );
        self.range_map(super::BSS_START, super::BSS_END, EntryBits::ReadWrite.val());
        self.range_map(
            super::KERNEL_STACK_START,
            super::KERNEL_STACK_END,
            EntryBits::ReadWrite.val(),
        );
        let num_pages = self.page_table.get_heap_pages_len();
        self.range_map(
            super::HEAP_START,
            super::HEAP_START + num_pages * PAGE_SIZE,
            EntryBits::ReadWrite.val(),
        );
        self.map(
            super::riscv64::MTIME_ADDRESS,
            super::riscv64::MTIME_ADDRESS,
            EntryBits::Read.val(),
            0,
        );
        self.map(
            super::riscv64::MTIMECMP_ADDRESS,
            super::riscv64::MTIMECMP_ADDRESS,
            EntryBits::ReadWrite.val(),
            0,
        );
        self.map(
            super::SIFIVE_TEST_ADDRESS,
            super::SIFIVE_TEST_ADDRESS,
            EntryBits::ReadWrite.val(),
            0,
        );
        // UART
        self.range_map(0x1000_0000, 0x1000_0000, EntryBits::ReadWrite.val());
        let self_addr = self as *const _ as usize;
        self.map(self_addr, self_addr, EntryBits::ReadWrite.val(), 0);
        // TODO Ver qué es esta dirección
        self.range_map(0x8009_4000, 0x8009_4000, EntryBits::ReadWrite.val());
        plic::map_pages(self);
        self.test_init_map();
    }

    /// Función para validar que esté todo mapeado
    /// (Olvidé mapear self.entries y perdí varias horas dandome cuenta)
    ///
    /// # Safety
    /// No tiene problemas de seguridad, las variables se inicializan por el linker
    pub unsafe fn test_init_map(&self) {
        let entry_address = &self.entries as *const _ as usize;
        let addresses = [
            (super::TEXT_START + 0x3500, "text_start"),
            (super::TEXT_END, "text_end"),
            (super::RODATA_START, "rodata_start"),
            (super::RODATA_END, "rodata_end"),
            (super::DATA_START, "data_start"),
            (super::DATA_END, "data_end"),
            (super::BSS_START, "bss_start"),
            (super::BSS_END, "bss_end"),
            (super::KERNEL_STACK_START, "kernel_stack_start"),
            (super::KERNEL_STACK_END, "ks_end"),
            (super::HEAP_START, "heap_start"),
            (entry_address, "entry_address"),
            (super::riscv64::MTIME_ADDRESS, "mtime address"),
            (0x80006df8, "core::fmt::write"),
        ];
        for address in &addresses {
            let phys = self.virt_to_phys(address.0).unwrap();
            println!(
                "Test walk {:20}: {:#x} -> {:#x}",
                address.1, address.0, phys
            );
        }
    }

    /// Armo el *satp*, con el PPN de la tabla y ASID 0
    /// El *supervisor address translation and protection register* esta formado por:
    /// * 4 bits de modo
    /// * 16 del address space identifier (ASID)
    /// * 44 de la dirección física de la raiz de la tabla de paginación (PPN)
    pub fn get_initial_satp(&self) -> usize {
        let asid = 1;
        let phys_addr = self as *const MapTable;
        let root_ppn = (phys_addr as i64) >> 12;
        let asid_bits = asid << 44;
        SATP_MODE_SV39 | asid_bits | root_ppn as usize
    }
}
