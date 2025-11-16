#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(asm_sym)]

use core::panic::PanicInfo;
use vga_driver::{Color, print_colored, clear_screen, println}; 

mod interrupts;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    clear_screen();
    
    println!("=== AerogelOS v0.1.0 ===");
    println!();
    
    print_colored("부팅 시퀀스 시작...\n", Color::LightCyan, Color::Black);
    println!();
    
    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("VGA 텍스트 드라이버 초기화");

    x86_64::instructions::interrupts::disable();
    
    // 1단계: IDT만 로드
    interrupts::init_idt();
    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("IDT 로드 완료");
    
    // 2단계: PIC 초기화 추가
    interrupts::init_pics();
    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("PIC 초기화 완료");
    
    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("커널 로드 완료");
    
    println!();
    print_colored("시스템 정보:\n", Color::Yellow, Color::Black);
    println!("  - 아키텍처: x86_64");
    println!("  - 개발자: 중학생 개발자!");
    println!("  - 환경: WSL2");
    
    println!();
    print_colored("커널 초기화 완료!\n", Color::LightGreen, Color::Black);
    
    // 3단계: 인터럽트 활성화
    print_colored("인터럽트 활성화 중...\n", Color::LightCyan, Color::Black);
    interrupts::enable_interrupts();
    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("CPU 인터럽트 활성화 완료");
    
    println!();
    print_colored("시스템 준비됨.\n", Color::LightCyan, Color::Black);
    
    println!();
    print_colored("시스템 준비됨. 키보드 폴링 모드:\n", Color::LightCyan, Color::Black);
    
    loop {
        if let Some(scancode) = interrupts::read_scancode() {
            if scancode & 0x80 == 0 {
                print_colored("K", Color::LightGreen, Color::Black);
            }
        }
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print_colored("\n\n!!! KERNEL PANIC !!!\n", Color::White, Color::Red);
    println!("{}", info);
    loop {
        x86_64::instructions::hlt();
    }
}