// kernel/src/main.rs

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::panic::PanicInfo;
use x86_64::instructions::hlt;
use x86_64::instructions::interrupts as x86_interrupts;

mod interrupts;
mod shell;
mod memory;

// ShellResult 대신 Shell만 가져옵니다 (경고 제거)
use shell::Shell;
// ShellResult는 아래 match문에서 shell::ShellResult::* 로 사용합니다.

#[no_mangle]
pub extern "C" fn _start() -> ! {
    clear_screen();
    vga_write(0, 0, "=== AerogelOS v0.1.0 ===", 0x0E);
    vga_write(0, 1, "Initializing heap...", 0x07);
    
    memory::init_heap();
    
    vga_write(0, 1, "Heap ready! Initializing interrupts...", 0x0A);
    
    interrupts::init_gdt();
    
    interrupts::init_idt();
    interrupts::init_pics();
    interrupts::enable_interrupts();
    
    vga_write(0, 1, "Interrupts ready! Type 'help' ", 0x0A);
    
    vga_write(0, 3, "> ", 0x0F);
    
    let mut shell = Shell::new();
    let mut current_row: usize = 3;
    
    loop {
		if let Some(scancode) = interrupts::read_scancode() {
			
			if scancode & 0x80 == 0 {
				
				if let Some(ch) = scancode_to_char(scancode) {
					if ch == '\n' {
						let result = shell.execute();
						
						current_row += 1;
						if current_row >= 24 {
							scroll_up();
							current_row = 23;
						}
						
						match result {
							shell::ShellResult::Clear => {
								clear_screen();
								vga_write(0, 0, "=== AerogelOS v0.1.0 ===", 0x0E);
								vga_write(0, 1, "Type 'help' for commands", 0x07);
								current_row = 3;
							},
							shell::ShellResult::Exit => {
								vga_write(0, current_row, "Shutting down...", 0x0C);
								use x86_64::instructions::port::Port;
								let mut port = Port::<u16>::new(0x604);
								unsafe { port.write(0x2000); }
								loop { hlt(); }
							},
							shell::ShellResult::MemTest => {
								use alloc::vec::Vec;
								use alloc::string::String;
								
								let mut test_vec = Vec::new();
								for i in 0..10 {
									test_vec.push(i);
								}
								
								let _test_string = String::from("Heap works!");
								
								vga_write(0, current_row, "Vec test: OK, String test: OK", 0x0A);
								current_row += 1;
								if current_row >= 24 {
									scroll_up();
									current_row = 23;
								}
							},
							shell::ShellResult::Output(text) => {
								vga_write(0, current_row, text, 0x0A);
								current_row += 1;
								if current_row >= 24 {
									scroll_up();
									current_row = 23;
								}
							},
							shell::ShellResult::Echo(buf, len) => {
								let text = core::str::from_utf8(&buf[..len]).unwrap_or("");
								vga_write(0, current_row, text, 0x0A);
								current_row += 1;
								if current_row >= 24 {
									scroll_up();
									current_row = 23;
								}
							},
							shell::ShellResult::Empty => {},
						}
						
						vga_write(0, current_row, "> ", 0x0F);
						
					} else if ch == '\x08' {
						shell.backspace();
						clear_line(current_row);
						vga_write(0, current_row, "> ", 0x0F);
						vga_write(2, current_row, shell.get_buffer(), 0x0F);
						
					} else {
						shell.add_char(ch);
						vga_write(2, current_row, shell.get_buffer(), 0x0F);
					}
				}
			}
		} else {
			hlt();
		}
	}
}

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
        for row in 1..25 {
            for col in 0..80 {
                let src_offset = ((row * 80 + col) * 2) as isize;
                let dst_offset = (((row - 1) * 80 + col) * 2) as isize;
                *vga.offset(dst_offset) = *vga.offset(src_offset);
                *vga.offset(dst_offset + 1) = *vga.offset(src_offset + 1);
            }
        }
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
        // 오류 수정: 00x1E -> 0x1E
        0x1E => Some('a'), 0x1F => Some('s'), 0x20 => Some('d'),
        0x21 => Some('f'), 0x22 => Some('g'), 0x23 => Some('h'),
        0x24 => Some('j'), 0x25 => Some('k'), 0x26 => Some('l'),
        0x2C => Some('z'), 0x2D => Some('x'), 0x2E => Some('c'),
        0x2F => Some('v'), 0x30 => Some('b'), 0x31 => Some('n'),
        0x32 => Some('m'),
        0x39 => Some(' '),
        0x1C => Some('\n'),
        0x0E => Some('\x08'),
        _ => None,
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    x86_interrupts::disable(); 
    vga_write(0, 20, "!!! KERNEL PANIC !!!", 0x4F);
    loop { hlt(); }
}