#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use vga_driver::{Color, print_colored, clear_screen, println}; 

mod interrupts;
mod pic;
mod keyboard;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    clear_screen();
    
    println!("=== AerogelOS v0.1.0 ===");
    println!();
    
    print_colored("부팅 시퀀스 시작...\n", Color::LightCyan, Color::Black);
    println!();
    
    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("VGA 텍스트 드라이버 초기화");
    
    // PIC 초기화 (인터럽트 컨트롤러)
    pic::init_pic();
    
    // IDT 초기화 (인터럽트 디스크립터 테이블)
    interrupts::init_idt();
    
    // 인터럽트 활성화
    x86_64::instructions::interrupts::enable();
    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("인터럽트 활성화");
    
    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("키보드 드라이버 초기화");
    
    println!();
    print_colored("시스템 정보:\n", Color::Yellow, Color::Black);
    println!("  - 아키텍처: x86_64");
    println!("  - 개발자: 중학생 개발자!");
    println!("  - 환경: WSL2");
    
    println!();
    print_colored("커널 초기화 완료!\n", Color::LightGreen, Color::Black);
    println!();
    print_colored("키보드 입력을 기다리는 중...\n", Color::LightCyan, Color::Black);
    print_colored("> ", Color::Yellow, Color::Black);
    
    // 무한 루프 (키보드 인터럽트 대기)
    loop {
        x86_64::instructions::hlt(); // CPU를 절전 모드로
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print_colored("\n\n!!! KERNEL PANIC !!!\n", Color::White, Color::Red);
    println!("{}", info);
    loop {}
}