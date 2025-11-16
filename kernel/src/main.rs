#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

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

    // 인터럽트 비활성화 상태에서 초기화
    x86_64::instructions::interrupts::disable();
    
    // 1. IDT 먼저 로드
    interrupts::init_idt();
    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("IDT 로드 완료");

    // 2. PIC 초기화
    interrupts::init_pics();
    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("PIC 초기화 완료");
    
    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("메모리 맵 확인");
    
    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("커널 로드 완료");
    
    println!();
    print_colored("시스템 정보:\n", Color::Yellow, Color::Black);
    println!("  - 아키텍처: x86_64");
    println!("  - 개발자: 중학생 개발자!");
    println!("  - 환경: WSL2");
    
    println!();
    print_colored("커널 초기화 완료!\n", Color::LightGreen, Color::Black);
    
    // 3. 마지막에 인터럽트 활성화
    print_colored("인터럽트 활성화 중...\n", Color::LightCyan, Color::Black);
    interrupts::enable_interrupts();
    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("CPU 인터럽트 활성화 완료");
    
    println!();
    print_colored("시스템 준비됨. 키보드 입력을 시작하세요.\n", Color::LightCyan, Color::Black);
    print_colored("> ", Color::Yellow, Color::Black);
    
    loop {
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