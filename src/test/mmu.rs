use crate::mmu::riscv64::PageBits;
use crate::mmu::HEAP_START;
use crate::PageTable;

/// Tests básicos de alloc y dealloc
#[test_case]
fn single_alloc() {
    // Test alloc simple
    let heap_start = unsafe { HEAP_START };
    let mut page_table = PageTable::new(heap_start, 0x8_0000);
    page_table.init();
    let ptr = page_table.alloc(1).unwrap();
    unsafe {
        assert_eq!(
            *(HEAP_START as *const u8),
            PageBits::Last.val() | PageBits::Used.val()
        )
    }
    page_table.dealloc(ptr);
    unsafe { assert_eq!(*(HEAP_START as *const u8), PageBits::Empty.val()) }
}

/// test multiples allocs
#[test_case]
fn multiple_alloc() {
    let heap_start = unsafe { HEAP_START };
    let mut page_table = PageTable::new(heap_start, 0x80_0000);
    page_table.init();
    let ptr = page_table.alloc(16).unwrap();
    for _ in 1..32 {
        page_table.alloc(16);
    }
    for i in 1..(32 * 16) {
        unsafe {
            if i % 16 == 15 {
                assert_eq!(
                    *((HEAP_START + i) as *const u8),
                    PageBits::Last.val() | PageBits::Used.val()
                )
            } else {
                assert_eq!(*((HEAP_START + i) as *const u8), PageBits::Used.val())
            }
        }
    }
    for i in 0..32 {
        unsafe {
            page_table.dealloc(ptr.add(16 * i * 4096));
        }
    }
    for i in 1..(32 * 16) {
        unsafe { assert_eq!(*((HEAP_START + i) as *const u8), PageBits::Empty.val()) }
    }
}
