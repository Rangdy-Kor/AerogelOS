// kernel/src/memory.rs - 안전한 버전
use linked_list_allocator::LockedHeap;
use core::alloc::Layout;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

// 더 큰 힙 공간
const HEAP_SIZE: usize = 200 * 1024; // 200 KiB
static mut HEAP_MEMORY: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

pub fn init_heap() {
    unsafe {
        let heap_start = HEAP_MEMORY.as_mut_ptr();
        ALLOCATOR.lock().init(heap_start, HEAP_SIZE);
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    // VGA에 직접 에러 출력
    let vga = 0xb8000 as *mut u16;
    let msg = b"ALLOC ERROR!";
    for (i, &byte) in msg.iter().enumerate() {
        unsafe {
            *vga.add(80 * 10 + i) = (byte as u16) | (0x4F << 8);
        }
    }
    
    // layout 정보도 출력
    let size = layout.size();
    let align = layout.align();
    
    // 간단한 16진수 출력
    fn write_hex(vga: *mut u16, offset: usize, val: usize) {
        const HEX: &[u8] = b"0123456789ABCDEF";
        for i in 0..8 {
            let digit = (val >> (28 - i * 4)) & 0xF;
            unsafe {
                *vga.add(offset + i) = (HEX[digit] as u16) | (0x4F << 8);
            }
        }
    }
    
    unsafe {
        let msg2 = b"Size:";
        for (i, &byte) in msg2.iter().enumerate() {
            *vga.add(80 * 11 + i) = (byte as u16) | (0x4F << 8);
        }
        write_hex(vga, 80 * 11 + 6, size);
        
        let msg3 = b"Align:";
        for (i, &byte) in msg3.iter().enumerate() {
            *vga.add(80 * 12 + i) = (byte as u16) | (0x4F << 8);
        }
        write_hex(vga, 80 * 12 + 7, align);
    }
    
    loop {
        x86_64::instructions::hlt();
    }
}