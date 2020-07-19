use crate::PageTable;
use crate::mmu::page_table;
use crate::mmu::page_table::PageBits;

/// Tests básicos de alloc y dealloc
#[test_case]
fn single_alloc() {
        // Test alloc simple
    let ptr = PageTable::alloc(1).unwrap();
    unsafe {
        assert_eq!(*(page_table::HEAP_START as *const u8),
            PageBits::Last.val() | PageBits::Used.val())
    }
    PageTable::dealloc(ptr);
    unsafe {
        assert_eq!(*(page_table::HEAP_START as *const u8),
            PageBits::Empty.val())
    }
}

/// test multiples allocs
#[test_case]
fn multiple_alloc() {
    let ptr = PageTable::alloc(16).unwrap();
    for _ in 1..32 {
        PageTable::alloc(16);
    }
    for i in 1..(32*16) {
        unsafe {
            if i % 16 == 15 {
                assert_eq!(*((page_table::HEAP_START + i) as *const u8),
                    PageBits::Last.val() | PageBits::Used.val())
            } else {
                assert_eq!(*((page_table::HEAP_START + i) as *const u8), PageBits::Used.val())
            }
        }
    }
    for i in 0..32 {
        unsafe {
            PageTable::dealloc(ptr.add(16 * i * 4096));
        }
    }
    for i in 1..(32*16) {
        unsafe {
            assert_eq!(*((page_table::HEAP_START + i) as *const u8), PageBits::Empty.val())
        }
    }
}
