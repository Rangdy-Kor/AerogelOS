#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use vga_driver::{Color, print_colored, clear_screen}; 
use vga_driver::println; 
use x86_64::instructions::port::Port;

mod interrupts;
mod keyboard;
mod pic;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    clear_screen();
    
	// PIC 초기화 추가
    print_colored("PIC 초기화 중...\n", Color::LightCyan, Color::Black);
    pic::init_pic(); // <--- PIC 초기화 함수 호출
    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("PIC 초기화 완료");

    // 다양한 색상으로 출력 테스트
    println!("=== AerogelOS v0.1.0 ===");
    println!();
    
    print_colored("부팅 시퀀스 시작...\n", Color::LightCyan, Color::Black);
    println!();
    
    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("VGA 텍스트 드라이버 초기화");

	interrupts::init_idt();

	// 0xFF (255) & ~(1 << 1) = 0xFD
	unsafe {
        let mut pic1_data = Port::<u8>::new(0x21); // PIC1 데이터 포트
        // IRQ 1(키보드)를 활성화 (나머지는 마스크 상태 유지)
        pic1_data.write(0xFD); 
    }
    
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
    print_colored("스크롤 테스트:\n", Color::Pink, Color::Black);
    for i in 0..30 {
        println!("라인 {}  - 스크롤이 작동하는지 확인", i);
    }
    
    println!();
    print_colored("색상 팔레트 테스트:", Color::White, Color::Black);
    println!();
    
    print_colored(" Black ", Color::Black, Color::White);
    print_colored(" Blue ", Color::Blue, Color::Black);
    print_colored(" Green ", Color::Green, Color::Black);
    print_colored(" Cyan ", Color::Cyan, Color::Black);
    println!();
    
    print_colored(" Red ", Color::Red, Color::Black);
    print_colored(" Magenta ", Color::Magenta, Color::Black);
    print_colored(" Brown ", Color::Brown, Color::Black);
    print_colored(" LightGray ", Color::LightGray, Color::Black);
    println!();
    
    print_colored(" DarkGray ", Color::DarkGray, Color::Black);
    print_colored(" LightBlue ", Color::LightBlue, Color::Black);
    print_colored(" LightGreen ", Color::LightGreen, Color::Black);
    print_colored(" LightCyan ", Color::LightCyan, Color::Black);
    println!();
    
    print_colored(" LightRed ", Color::LightRed, Color::Black);
    print_colored(" Pink ", Color::Pink, Color::Black);
    print_colored(" Yellow ", Color::Yellow, Color::Black);
    print_colored(" White ", Color::White, Color::Black);
    println!();
    
    println!();
    print_colored("커널 초기화 완료!\n", Color::LightGreen, Color::Black);
    print_colored("시스템 준비됨.\n", Color::LightCyan, Color::Black);
    
    loop {
        // x86 명령어 'hlt' (halt)를 실행하여 CPU를 낮은 전력 상태로 멈춥니다.
        // 다음 인터럽트가 발생할 때까지 CPU는 멈춰있습니다.
        // 만약 'hlt'를 지원하지 않는다면 단순 루프도 재부팅은 막을 수 있습니다.
        x86_64::instructions::hlt(); 
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print_colored("\n\n!!! KERNEL PANIC !!!\n", Color::White, Color::Red);
    println!("{}", info);
    loop {}
}