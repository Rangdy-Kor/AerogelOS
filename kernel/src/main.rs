#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use core::fmt::Write;

mod interrupts;

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

fn clear_screen() {
    let vga = 0xb8000 as *mut u8;
    for i in 0..(80 * 25) {
        unsafe {
            *vga.offset((i * 2) as isize) = b' ';
            *vga.offset((i * 2 + 1) as isize) = 0x0F;
        }
    }
}

fn scancode_to_char(scancode: u8) -> Option<char> {
    match scancode {
        0x02 => Some('1'), 0x03 => Some('2'), 0x04 => Some('3'),
        0x05 => Some('4'), 0x06 => Some('5'), 0x07 => Some('6'),
        0x08 => Some('7'), 0x09 => Some('8'), 0x0A => Some('9'),
        0x0B => Some('0'),
        0x10 => Some('q'), 0x11 => Some('w'), 0x12 => Some('e'),
        0x13 => Some('r'), 0x14 => Some('t'), 0x15 => Some('y'),
        0x16 => Some('u'), 0x17 => Some('i'), 0x18 => Some('o'),
        0x19 => Some('p'),
        0x1E => Some('a'), 0x1F => Some('s'), 0x20 => Some('d'),
        0x21 => Some('f'), 0x22 => Some('g'), 0x23 => Some('h'),
        0x24 => Some('j'), 0x25 => Some('k'), 0x26 => Some('l'),
        0x2C => Some('z'), 0x2D => Some('x'), 0x2E => Some('c'),
        0x2F => Some('v'), 0x30 => Some('b'), 0x31 => Some('n'),
        0x32 => Some('m'),
        0x39 => Some(' '),
        0x1C => Some('\n'),
        _ => None,
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    clear_screen();
    
    vga_write(0, 0, "=== AerogelOS v0.1.0 ===", 0x0E);
    vga_write(0, 1, "Polling Mode (No Interrupts)", 0x07);
    vga_write(0, 3, "Type something:", 0x0F);
    
    x86_64::instructions::interrupts::disable();
    interrupts::init_gdt();
    
    let vga = 0xb8000 as *mut u8;
    let mut input_pos = 0;
    let input_row = 5;
    
    loop {
        use x86_64::instructions::port::Port;
        let mut status_port = Port::<u8>::new(0x64);
        let mut data_port = Port::<u8>::new(0x60);
        
        unsafe {
            let status = status_port.read();
            if status & 0x01 != 0 {
                let scancode = data_port.read();
                
                // 키 눌림만 처리 (release는 무시)
                if scancode & 0x80 == 0 {
                    if let Some(ch) = scancode_to_char(scancode) {
                        if ch == '\n' {
                            input_pos = 0;
                            // 줄 지우기
                            for i in 0..80 {
                                *vga.offset((80 * input_row + i) * 2) = b' ';
                                *vga.offset((80 * input_row + i) * 2 + 1) = 0x0F;
                            }
                        } else if input_pos < 80 {
                            *vga.offset((80 * input_row + input_pos) * 2) = ch as u8;
                            *vga.offset((80 * input_row + input_pos) * 2 + 1) = 0x0A;
                            input_pos += 1;
                        }
                    }
                }
            }
        }
        
        // CPU 휴식
        for _ in 0..1000 {
            unsafe { core::arch::asm!("nop"); }
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    vga_write(0, 20, "!!! KERNEL PANIC !!!", 0x4F);
    loop { x86_64::instructions::hlt(); }
}