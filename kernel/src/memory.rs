// kernel/src/memory.rs
use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

// 간단한 정적 힙 (스택에 가까운 위치)
const HEAP_SIZE: usize = 100 * 1024; // 100 KiB
static mut HEAP_MEMORY: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

pub fn init_heap() {
    unsafe {
        let heap_start = HEAP_MEMORY.as_ptr() as usize;
        ALLOCATOR.lock().init(heap_start as *mut u8, HEAP_SIZE);
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}