#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

mod interrupts;
mod shell;

use shell::{Shell, ShellResult};

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

fn clear_line(row: usize) {
    let vga = 0xb8000 as *mut u8;
    for i in 0..80 {
        unsafe {
            *vga.offset(((80 * row + i) * 2) as isize) = b' ';
            *vga.offset(((80 * row + i) * 2 + 1) as isize) = 0x0F;
        }
    }
}

fn scroll_up() {
    let vga = 0xb8000 as *mut u8;
    unsafe {
        // 1행부터 마지막 행까지를 한 줄씩 위로 복사
        for row in 1..25 {
            for col in 0..80 {
                let src_offset = ((row * 80 + col) * 2) as isize;
                let dst_offset = (((row - 1) * 80 + col) * 2) as isize;
                *vga.offset(dst_offset) = *vga.offset(src_offset);
                *vga.offset(dst_offset + 1) = *vga.offset(src_offset + 1);
            }
        }
        // 마지막 줄 지우기
        for col in 0..80 {
            *vga.offset(((24 * 80 + col) * 2) as isize) = b' ';
            *vga.offset(((24 * 80 + col) * 2 + 1) as isize) = 0x0F;
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
        0x0E => Some('\x08'), // Backspace
        _ => None,
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    clear_screen();
    
    vga_write(0, 0, "=== AerogelOS v0.1.0 ===", 0x0E);
    vga_write(0, 1, "Type 'help' for commands", 0x07);
    vga_write(0, 2, "", 0x0F);
    vga_write(0, 3, "> ", 0x0F);
    
    x86_64::instructions::interrupts::disable();
    interrupts::init_gdt();
    
    let mut shell = Shell::new();
    let mut current_row: usize = 3;
    
    loop {
        use x86_64::instructions::port::Port;
        let mut status_port = Port::<u8>::new(0x64);
        let mut data_port = Port::<u8>::new(0x60);
        
        unsafe {
            let status = status_port.read();
            if status & 0x01 != 0 {
                let scancode = data_port.read();
                
                if scancode & 0x80 == 0 {
                    if let Some(ch) = scancode_to_char(scancode) {
                        if ch == '\n' {
                            // 명령어 실행
                            let result = shell.execute();
                            
                            // 다음 줄로 이동
                            current_row += 1;
                            if current_row >= 24 {
                                scroll_up();
                                current_row = 23;
                            }
                            
                            match result {
                                ShellResult::Clear => {
                                    clear_screen();
                                    vga_write(0, 0, "=== AerogelOS v0.1.0 ===", 0x0E);
                                    vga_write(0, 1, "Type 'help' for commands", 0x07);
                                    current_row = 3;
                                },
                                ShellResult::Output(text) => {
                                    vga_write(0, current_row, text, 0x0A);
                                    current_row += 1;
                                    if current_row >= 24 {
                                        scroll_up();
                                        current_row = 23;
                                    }
                                },
                                ShellResult::Echo(buf, len) => {
                                    let text = core::str::from_utf8(&buf[..len]).unwrap_or("");
                                    vga_write(0, current_row, text, 0x0A);
                                    current_row += 1;
                                    if current_row >= 24 {
                                        scroll_up();
                                        current_row = 23;
                                    }
                                },
                                ShellResult::Empty => {},
                            }
                            
                            // 새 프롬프트
                            vga_write(0, current_row, "> ", 0x0F);
                            
                        } else if ch == '\x08' {
                            // Backspace
                            shell.backspace();
                            clear_line(current_row);
                            vga_write(0, current_row, "> ", 0x0F);
                            vga_write(2, current_row, shell.get_buffer(), 0x0F);
                            
                        } else {
                            // 일반 문자
                            shell.add_char(ch);
                            vga_write(2, current_row, shell.get_buffer(), 0x0F);
                        }
                    }
                }
            }
        }
        
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