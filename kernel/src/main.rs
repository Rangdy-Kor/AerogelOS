// kernel/src/main.rs - 확장된 명령어 지원
// kernel/src/main.rs
#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)] 

extern crate alloc;

use core::panic::PanicInfo;
use x86_64::instructions::hlt;
use x86_64::instructions::port::Port;

mod shell;
mod memory;
mod interrupts;

use shell::Shell;


static mut TICK_COUNTER: u64 = 0;
static mut BG_COLOR: u8 = 0x0;

#[no_mangle]
pub fn increment_tick() {
    unsafe { TICK_COUNTER = TICK_COUNTER.wrapping_add(1); }
}

// PIT 초기화 함수 추가
fn init_pit() {
    unsafe {
        // PIT를 100Hz로 설정 (10ms마다 인터럽트)
        let divisor: u16 = 11932; // 1193182 Hz / 100 Hz
        let mut command_port = Port::<u8>::new(0x43);
        let mut data_port = Port::<u8>::new(0x40);
        
        command_port.write(0x36); // Channel 0, lobyte/hibyte, rate generator
        data_port.write((divisor & 0xFF) as u8);
        data_port.write((divisor >> 8) as u8);
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    clear_screen();
    vga_write(0, 0, "=== AerogelOS v0.1.0 ===", 0x0E);
    vga_write(0, 1, "[1/4] Initializing GDT...", 0x07);
    
    interrupts::init_gdt();
    vga_write(0, 1, "[2/4] Initializing IDT...    ", 0x07);
    
    interrupts::init_idt();
    vga_write(0, 1, "[3/4] Initializing heap...   ", 0x07);
    
    memory::init_heap();
    vga_write(0, 1, "[4/4] Starting interrupts... ", 0x07);
    
    init_pit(); // PIT 초기화
    interrupts::init_pics();
    interrupts::enable_interrupts();
    
    vga_write(0, 1, "[DONE] System ready!          ", 0x0A);
    vga_write(0, 2, "Welcome to AerogelOS!", 0x0F);
    vga_write(0, 3, "Type 'help' for available commands", 0x07);
    vga_write(0, 5, "> ", 0x0F);
    
    let mut shell = Shell::new();
    shell.set_boot_time(unsafe { TICK_COUNTER });
    let mut current_row: usize = 5;
    
    let mut status_port = Port::<u8>::new(0x64);
    let mut data_port = Port::<u8>::new(0x60);
    
    // 초기 키보드 버퍼 비우기
    for _ in 0..10 {
        let status: u8 = unsafe { status_port.read() };
        if (status & 0x01) != 0 {
            let _: u8 = unsafe { data_port.read() };
        }
    }
    
    loop {
        let status: u8 = unsafe { status_port.read() };
        
        if (status & 0x01) != 0 {
            let scancode: u8 = unsafe { data_port.read() };
            
            if (scancode & 0x80) == 0 {
                if let Some(ch) = scancode_to_char(scancode) {
                    if ch == '\n' {
                        let result = shell.execute(unsafe { TICK_COUNTER });
                        
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
                            shell::ShellResult::Shutdown => {
                                clear_screen();
                                vga_write(30, 12, "Shutting down...", 0x0C);
                                let mut port = Port::<u16>::new(0x604);
                                unsafe { port.write(0x2000); }
                                loop { hlt(); }
                            },
                            shell::ShellResult::Reboot => {
                                clear_screen();
                                vga_write(32, 12, "Rebooting...", 0x0E);
                                unsafe {
                                    let mut port = Port::<u8>::new(0x64);
                                    port.write(0xFE);
                                }
                                loop { hlt(); }
                            },
                            shell::ShellResult::CpuInfo => {
                                vga_write(0, current_row, "CPU: x86_64 compatible", 0x0B);
                                current_row += 1;
                                vga_write(0, current_row, "Vendor: (Use CPUID for details)", 0x07);
                                current_row += 1;
                                if current_row >= 24 { scroll_up(); current_row = 23; }
                            },
                            shell::ShellResult::MemInfo => {
                                use alloc::vec::Vec;
                                use alloc::string::String;
                                
                                vga_write(0, current_row, "Testing memory allocator...", 0x0E);
                                current_row += 1;
                                if current_row >= 24 { scroll_up(); current_row = 23; }
                                
                                let mut test_vec = Vec::new();
                                for i in 0..10 {
                                    test_vec.push(i * 10);
                                }
                                let test_string = String::from("Heap OK!");
                                
                                vga_write(0, current_row, "Heap: 200KB allocated, Status: OK", 0x0A);
                                current_row += 1;
                                if current_row >= 24 { scroll_up(); current_row = 23; }
                                
                                drop(test_vec);
                                drop(test_string);
                            },
                            shell::ShellResult::SysInfo => {
                                vga_write(0, current_row, "=== System Information ===", 0x0E);
                                current_row += 1;
                                if current_row >= 24 { scroll_up(); current_row = 23; }
                                
                                vga_write(0, current_row, "OS: AerogelOS v0.1.0", 0x0B);
                                current_row += 1;
                                if current_row >= 24 { scroll_up(); current_row = 23; }
                                
                                vga_write(0, current_row, "Architecture: x86_64", 0x0B);
                                current_row += 1;
                                if current_row >= 24 { scroll_up(); current_row = 23; }
                                
                                vga_write(0, current_row, "Memory: 200KB heap", 0x0B);
                                current_row += 1;
                                if current_row >= 24 { scroll_up(); current_row = 23; }
                                
                                vga_write(0, current_row, "Timer: 100Hz PIT", 0x0B);
                                current_row += 1;
                                if current_row >= 24 { scroll_up(); current_row = 23; }
                            },
                            shell::ShellResult::DateTime => {
                                vga_write(0, current_row, "Date: 2025-01-XX (RTC not impl)", 0x0B);
                                current_row += 1;
                                if current_row >= 24 { scroll_up(); current_row = 23; }
                                
                                vga_write(0, current_row, "Time: HH:MM:SS (RTC not impl)", 0x0B);
                                current_row += 1;
                                if current_row >= 24 { scroll_up(); current_row = 23; }
                            },
                            shell::ShellResult::Uptime(ticks) => {
                                // 100Hz PIT이므로 1초 = 100 틱
                                let seconds = ticks / 100;
                                let minutes = seconds / 60;
                                let hours = minutes / 60;
                                
                                let mut buf = [0u8; 64];
                                let text = format_uptime(hours, minutes % 60, seconds % 60, &mut buf);
                                vga_write(0, current_row, text, 0x0B);
                                current_row += 1;
                                if current_row >= 24 { scroll_up(); current_row = 23; }
                            },
                            shell::ShellResult::BgColor(color) => {
                                unsafe { BG_COLOR = color; }
                                change_background(color);
                                vga_write(0, current_row, "Background color changed!", 0x0A);
                                current_row += 1;
                                if current_row >= 24 { scroll_up(); current_row = 23; }
                            },
                            shell::ShellResult::Output(text) => {
                                vga_write(0, current_row, text, 0x0A);
                                current_row += 1;
                                if current_row >= 24 { scroll_up(); current_row = 23; }
                            },
                            shell::ShellResult::MultiOutput(lines, count) => {
                                for i in 0..count {
                                    vga_write(0, current_row, lines[i], 0x0B);
                                    current_row += 1;
                                    if current_row >= 24 { scroll_up(); current_row = 23; }
                                }
                            },
                            shell::ShellResult::Print(buf, len) => {
                                let text = core::str::from_utf8(&buf[..len]).unwrap_or("");
                                vga_write(0, current_row, text, 0x0F);
                                current_row += 1;
                                if current_row >= 24 { scroll_up(); current_row = 23; }
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
                        clear_line(current_row);
                        vga_write(0, current_row, "> ", 0x0F);
                        vga_write(2, current_row, shell.get_buffer(), 0x0F);
                    }
                }
            }
        }

		// hlt() 제거 - 계속 폴링
		for _ in 0..500 {
			unsafe { core::arch::asm!("pause"); }
		}
        
        // hlt(); // CPU를 절전 모드로 전환 (인터럽트 대기)
    }
}

fn format_uptime(hours: u64, minutes: u64, seconds: u64, buf: &mut [u8]) -> &str {
    let mut pos = 0;
    let prefix = b"Uptime: ";
    for &b in prefix {
        buf[pos] = b;
        pos += 1;
    }
    
    // 시간
    if hours > 0 {
        pos += write_num(&mut buf[pos..], hours);
        buf[pos] = b'h';
        pos += 1;
        buf[pos] = b' ';
        pos += 1;
    }
    
    // 분
    if minutes > 0 || hours > 0 {
        pos += write_num(&mut buf[pos..], minutes);
        buf[pos] = b'm';
        pos += 1;
        buf[pos] = b' ';
        pos += 1;
    }
    
    // 초
    pos += write_num(&mut buf[pos..], seconds);
    buf[pos] = b's';
    pos += 1;
    
    core::str::from_utf8(&buf[..pos]).unwrap_or("Uptime: N/A")
}

fn write_num(buf: &mut [u8], mut num: u64) -> usize {
    if num == 0 {
        buf[0] = b'0';
        return 1;
    }
    
    let mut digits = [0u8; 20];
    let mut count = 0;
    while num > 0 {
        digits[count] = (num % 10) as u8 + b'0';
        num /= 10;
        count += 1;
    }
    
    for i in 0..count {
        buf[i] = digits[count - 1 - i];
    }
    count
}

fn change_background(color: u8) {
    let vga = 0xb8000 as *mut u8;
    for i in 0..(80 * 25) {
        unsafe {
            let _current_char = *vga.offset((i * 2) as isize);
            let current_fg = *vga.offset((i * 2 + 1) as isize) & 0x0F;
            *vga.offset((i * 2 + 1) as isize) = (color << 4) | current_fg;
        }
    }
}

fn vga_write(x: usize, y: usize, s: &str, color: u8) {
    let vga = 0xb8000 as *mut u8;
    let offset = (y * 80 + x) * 2;
    let bg = unsafe { BG_COLOR };
    let full_color = (bg << 4) | color;
    
    for (i, byte) in s.bytes().enumerate() {
        if x + i >= 80 { break; }
        unsafe {
            *vga.offset((offset + i * 2) as isize) = byte;
            *vga.offset((offset + i * 2 + 1) as isize) = full_color;
        }
    }
}

fn clear_screen() {
    let vga = 0xb8000 as *mut u8;
    let bg = unsafe { BG_COLOR };
    for i in 0..(80 * 25) {
        unsafe {
            *vga.offset((i * 2) as isize) = b' ';
            *vga.offset((i * 2) as isize + 1) = (bg << 4) | 0x0F;
        }
    }
}

fn clear_line(row: usize) {
    let vga = 0xb8000 as *mut u8;
    let bg = unsafe { BG_COLOR };
    for i in 0..80 {
        unsafe {
            *vga.offset(((80 * row + i) * 2) as isize) = b' ';
            *vga.offset(((80 * row + i) * 2 + 1) as isize) = (bg << 4) | 0x0F;
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
        let bg = BG_COLOR;
        for col in 0..80 {
            *vga.offset(((24 * 80 + col) * 2) as isize) = b' ';
            *vga.offset(((24 * 80 + col) * 2 + 1) as isize) = (bg << 4) | 0x0F;
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
        0x0E => Some('\x08'),
        _ => None,
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let vga = 0xb8000 as *mut u16;
    for i in 0..(80 * 25) {
        unsafe {
            *vga.add(i) = (b' ' as u16) | (0x4F << 8);
        }
    }
    let msg = b"!!! KERNEL PANIC !!!";
    for (i, &byte) in msg.iter().enumerate() {
        unsafe {
            *vga.add(80 * 10 + 30 + i) = (byte as u16) | (0x4F << 8);
        }
    }
    loop { hlt(); }
}