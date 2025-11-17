// 7단계: 루프 진#![no_std]
#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

mod interrupts;

// 직접 VGA 쓰기 (디버그용)
fn vga_write(x: usize, y: usize, s: &str, color: u8) {
    let vga = 0xb8000 as *mut u8;
    let offset = (y * 80 + x) * 2;
    for (i, byte) in s.bytes().enumerate() {
        unsafe {
            *vga.offset((offset + i * 2) as isize) = byte;
            *vga.offset((offset + i * 2 + 1) as isize) = color;
        }
    }
}

// 직접 VGA로 점 출력
fn print_dot(pos: usize) {
    let vga = 0xb8000 as *mut u8;
    unsafe {
        *vga.offset((pos * 2) as isize) = b'.';
        *vga.offset((pos * 2 + 1) as isize) = 0x0E; // 노란색
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // 1단계: 시작 표시
    vga_write(0, 0, "STEP 1: Start", 0x0F);
    
    // 2단계: 인터럽트 비활성화
    x86_64::instructions::interrupts::disable();
    vga_write(0, 1, "STEP 2: Disabled interrupts", 0x0F);
    
    // 3단계: GDT 초기화
    interrupts::init_gdt();
    vga_write(0, 2, "STEP 3: GDT loaded", 0x0F);
    
    // 4단계: IDT 초기화
    interrupts::init_idt();
    vga_write(0, 3, "STEP 4: IDT loaded", 0x0F);
    
    // 5단계: PIC 초기화
    interrupts::init_pics();
    vga_write(0, 4, "STEP 5: PIC initialized", 0x0F);
    
    // 5.5단계: PIC 초기화 후 대기
    for _ in 0..10000 {
        unsafe { core::arch::asm!("nop"); }
    }
    vga_write(0, 5, "STEP 5.5: PIC ready", 0x0F);
    
    // 6단계: 인터럽트 활성화
    interrupts::enable_interrupts();
    vga_write(0, 6, "STEP 6: Interrupts enabled", 0x0F);
    
    // 6.5단계: 인터럽트 상태 확인
    if interrupts::are_interrupts_enabled() {
        vga_write(0, 7, "STEP 6.5: IF=1 (OK)", 0x0A); // 녹색
    } else {
        vga_write(0, 7, "STEP 6.5: IF=0 (ERROR)", 0x0C); // 빨간색
    }
    
    // 7단계: 루프 진입
    vga_write(0, 8, "STEP 7: Entering loop", 0x0F);
    
    // 라벨 미리 출력
    vga_write(0, 10, "Counter:", 0x0F);
    vga_write(0, 11, "Timer:", 0x0E);
    vga_write(0, 12, "Keyboard:", 0x0B);
    
    // 무한 루프 - 최대한 단순화
    let mut counter: u32 = 0;
    let vga = 0xb8000 as *mut u8;
    
    loop {
        counter = counter.wrapping_add(1);
        
        // 비트 마스크로 체크 (65536번마다)
        if counter & 0xFFFF == 0 {
            unsafe {
                // 간단히 'X' 출력
                *vga.offset((80 * 10 + 9) * 2) = b'X';
                *vga.offset((80 * 10 + 9) * 2 + 1) = 0x0F;
            }
        }
        
        // 빈 루프 (최적화 방지)
        unsafe { core::arch::asm!("nop"); }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    vga_write(0, 10, "!!! KERNEL PANIC !!!", 0x4F); // 빨간 배경
    loop {
        x86_64::instructions::hlt();
    }
}