#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use vga_driver::{Color, print_colored, clear_screen, println}; 

mod interrupts;

fn print_hex_byte(byte: u8) {
    const HEX_CHARS: &[u8] = b"0123456789ABCDEF";
    print_colored("0x", Color::Yellow, Color::Black);
    
    let high = (byte >> 4) as usize;
    let low = (byte & 0x0F) as usize;
    
    let mut buf = [0u8; 1];
    buf[0] = HEX_CHARS[high];
    if let Ok(s) = core::str::from_utf8(&buf) {
        print_colored(s, Color::Yellow, Color::Black);
    }
    buf[0] = HEX_CHARS[low];
    if let Ok(s) = core::str::from_utf8(&buf) {
        print_colored(s, Color::Yellow, Color::Black);
    }
}

fn handle_keypress(scancode: u8) {
    let key = match scancode {
        0x01 => Some("ESC"),
        0x02 => Some("1"),
        0x03 => Some("2"),
        0x04 => Some("3"),
        0x05 => Some("4"),
        0x06 => Some("5"),
        0x07 => Some("6"),
        0x08 => Some("7"),
        0x09 => Some("8"),
        0x0A => Some("9"),
        0x0B => Some("0"),
        0x0C => Some("-"),
        0x0D => Some("="),
        0x0E => Some("Backspace"),
        0x0F => Some("Tab"),
        0x10 => Some("Q"),
        0x11 => Some("W"),
        0x12 => Some("E"),
        0x13 => Some("R"),
        0x14 => Some("T"),
        0x15 => Some("Y"),
        0x16 => Some("U"),
        0x17 => Some("I"),
        0x18 => Some("O"),
        0x19 => Some("P"),
        0x1A => Some("["),
        0x1B => Some("]"),
        0x1C => Some("Enter"),
        0x1D => Some("LCtrl"),
        0x1E => Some("A"),
        0x1F => Some("S"),
        0x20 => Some("D"),
        0x21 => Some("F"),
        0x22 => Some("G"),
        0x23 => Some("H"),
        0x24 => Some("J"),
        0x25 => Some("K"),
        0x26 => Some("L"),
        0x27 => Some(";"),
        0x28 => Some("'"),
        0x29 => Some("`"),
        0x2A => Some("LShift"),
        0x2B => Some("\\"),
        0x2C => Some("Z"),
        0x2D => Some("X"),
        0x2E => Some("C"),
        0x2F => Some("V"),
        0x30 => Some("B"),
        0x31 => Some("N"),
        0x32 => Some("M"),
        0x33 => Some(","),
        0x34 => Some("."),
        0x35 => Some("/"),
        0x36 => Some("RShift"),
        0x38 => Some("LAlt"),
        0x39 => Some("Space"),
        _ => None,
    };
    
    if let Some(key_name) = key {
        print_colored("[", Color::DarkGray, Color::Black);
        print_colored(key_name, Color::LightGreen, Color::Black);
        print_colored("] ", Color::DarkGray, Color::Black);
    } else {
        print_colored("[", Color::DarkGray, Color::Black);
        print_hex_byte(scancode);
        print_colored("] ", Color::DarkGray, Color::Black);
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // 인터럽트 비활성화
    x86_64::instructions::interrupts::disable();
    
    clear_screen();
    
    println!("=== AerogelOS v0.1.0 DEBUG ===");
    println!();
    
    print_colored("부팅 시퀀스 시작...\n", Color::LightCyan, Color::Black);
    println!();
    
    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("VGA 텍스트 드라이버 초기화");

    // GDT 초기화
    print_colored("[...] ", Color::Yellow, Color::Black);
    println!("GDT 초기화 중...");
    interrupts::init_gdt();
    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("GDT 로드 완료");
    
    // IDT 초기화
    print_colored("[...] ", Color::Yellow, Color::Black);
    println!("IDT 초기화 중...");
    interrupts::init_idt();
    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("IDT 로드 완료");
    
    // PIC 초기화
    print_colored("[...] ", Color::Yellow, Color::Black);
    println!("PIC 초기화 중...");
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
    
    // 인터럽트 플래그 확인
    print_colored("[TEST] ", Color::Cyan, Color::Black);
    println!("인터럽트 활성화 전 - CPU 플래그 확인");
    
    // 인터럽트 활성화
    print_colored("[...] ", Color::Yellow, Color::Black);
    println!("CPU 인터럽트 활성화 중...");
    interrupts::enable_interrupts();
    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("CPU 인터럽트 활성화 완료");
    
    // 인터럽트가 활성화됐는지 확인
    print_colored("[TEST] ", Color::Cyan, Color::Black);
    println!("인터럽트 플래그 확인...");
    
    println!();
    println!("키보드 입력 대기 중... (타이머 인터럽트로 점이 찍혀야 함)");
    println!();
    
    // 카운터로 루프가 도는지 확인
    let mut counter: u32 = 0;
    
    loop {
        counter = counter.wrapping_add(1);
        
        // 1000000번마다 수동으로 메시지 출력
        if counter % 1000000 == 0 {
            print_colored("L", Color::Cyan, Color::Black);  // Loop 표시
        }
        
        // 버퍼에서 스캔코드 읽기
        if let Some(scancode) = interrupts::read_scancode() {
            println!();
            print_colored("[SCAN] ", Color::Magenta, Color::Black);
            // 키 눌림만 처리
            if scancode & 0x80 == 0 {
                handle_keypress(scancode);
            }
            println!();
        }
        
        // 타이머 인터럽트가 CPU를 깨워줌
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