use crate::mmu::map_table::MapTable;
use crate::mmu::map_table::EntryBits;
use crate::mmu::page_table::PageTable;

#[test_case]
fn identity_single_map() {
    let map_table_page = PageTable::zalloc(1).unwrap();
    let map_table : &mut MapTable = unsafe {&mut *(map_table_page as *mut MapTable)};
    map_table.map(0, 0, EntryBits::Read.val(), 0);
    map_table.map(0x1337000, 0x1337000, EntryBits::Read.val(), 0);
    map_table.map(0xFFFF_FFFF_FFFF_F000, 0xFFFF_FFFF_FFFF_F000, EntryBits::Read.val(), 0);
    assert_eq!(map_table.virt_to_phys(0), Some(0));
    assert_eq!(map_table.virt_to_phys(0x123), Some(0x123));
    assert_eq!(map_table.virt_to_phys(0x1337000), Some(0x1337000));
    assert_eq!(map_table.virt_to_phys(0x1337FFF), Some(0x1337FFF));
    assert_eq!(map_table.virt_to_phys(0xFFFF_FFFF_FFFF_F000), Some(0xFFFF_FFFF_FFFF_F000));
    // Unmap
    map_table.unmap();
    assert_eq!(map_table.virt_to_phys(0), None);
    assert_eq!(map_table.virt_to_phys(0x1337000), None);
    assert_eq!(map_table.virt_to_phys(0xFFFF_FFFF_FFFF_F000), None);
    PageTable::dealloc(map_table_page);
}

#[test_case]
fn identity_range_map() {
	let map_table_page = PageTable::zalloc(1).unwrap();
    let map_table : &mut MapTable = unsafe {&mut *(map_table_page as *mut MapTable)};
    map_table.identity_map(0x1337000, 0x1340000, EntryBits::Read.val());
    assert_eq!(map_table.virt_to_phys(0x1338000), Some(0x1338000));
    assert_eq!(map_table.virt_to_phys(0x1340000), Some(0x1340000));
    assert_eq!(map_table.virt_to_phys(0x1341000), None);
    PageTable::dealloc(map_table_page);
}
