#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::panic::PanicInfo;

mod interrupts;
mod shell;
mod memory;

use shell::{Shell, ShellResult};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    clear_screen();
    vga_write(0, 0, "=== AerogelOS v0.1.0 - Debug Mode ===", 0x0E);
    vga_write(0, 1, "[1/6] Initializing heap...", 0x07);
    
    memory::init_heap();
    vga_write(0, 1, "[1/6] Heap ready!                    ", 0x0A);
    
    vga_write(0, 2, "[2/6] Loading GDT...", 0x07);
    interrupts::init_gdt();
    vga_write(0, 2, "[2/6] GDT loaded!                    ", 0x0A);
    
    vga_write(0, 3, "[3/6] Loading IDT...", 0x07);
    interrupts::init_idt();
    vga_write(0, 3, "[3/6] IDT loaded!                    ", 0x0A);
    
    vga_write(0, 4, "[4/6] Initializing PIC...", 0x07);
    interrupts::init_pics();
    vga_write(0, 4, "[4/6] PIC initialized!               ", 0x0A);
    
    vga_write(0, 5, "[5/6] Enabling interrupts...", 0x07);
    interrupts::enable_interrupts();
    
    if interrupts::are_interrupts_enabled() {
        vga_write(0, 5, "[5/6] Interrupts ENABLED!            ", 0x0A);
    } else {
        vga_write(0, 5, "[5/6] ERROR: Interrupts NOT enabled! ", 0x0C);
    }
    
    vga_write(0, 6, "[6/6] Testing interrupts (150 timer ticks)...", 0x07);
    vga_write(0, 7, "Press keys to test keyboard!", 0x0B);
    
    // 타이머 인터럽트 기반 대기 (약 3초 = 150 ticks @ 18.2Hz)
    let start_ticks = interrupts::get_timer_ticks();
    let target_ticks = start_ticks + 150;
    
    loop {
        let current_ticks = interrupts::get_timer_ticks();
        let kb_interrupts = interrupts::get_keyboard_interrupts();
        
        // 타이머 표시
        let mut timer_str = [b' '; 50];
        write_msg(&mut timer_str, b"Timer: ");
        write_u64_at(&mut timer_str, 7, current_ticks);
        vga_write(0, 9, core::str::from_utf8(&timer_str[..17]).unwrap_or(""), 0x0F);
        
        // 키보드 표시
        let mut kb_str = [b' '; 50];
        write_msg(&mut kb_str, b"Keyboard: ");
        write_u64_at(&mut kb_str, 10, kb_interrupts);
        vga_write(0, 10, core::str::from_utf8(&kb_str[..20]).unwrap_or(""), 0x0F);
        
        if current_ticks >= target_ticks {
            break;
        }
        
        x86_64::instructions::hlt();
    }
    
    // 결과 분석
    let final_timer = interrupts::get_timer_ticks();
    let final_kb = interrupts::get_keyboard_interrupts();
    
    vga_write(0, 12, "=== Test Results ===", 0x0E);
    
    if final_timer > start_ticks {
        vga_write(0, 13, "[OK] Timer interrupts working!", 0x0A);
    } else {
        vga_write(0, 13, "[FAIL] Timer interrupts not working", 0x0C);
        vga_write(0, 14, "Cannot continue without timer", 0x0C);
        loop { x86_64::instructions::hlt(); }
    }
    
    if final_kb > 0 {
        vga_write(0, 14, "[OK] Keyboard interrupts working!", 0x0A);
    } else {
        vga_write(0, 14, "[WARN] No keyboard input detected", 0x0E);
    }
    
    vga_write(0, 16, "Starting shell...", 0x0F);
    
    // 약 1초 대기
    let wait_until = final_timer + 18;
    while interrupts::get_timer_ticks() < wait_until {
        x86_64::instructions::hlt();
    }
    
    // 쉘 시작
    clear_screen();
    vga_write(0, 0, "=== AerogelOS Shell ===", 0x0E);
    
    if final_kb > 0 {
        vga_write(0, 1, "Mode: INTERRUPT-BASED", 0x0A);
    } else {
        vga_write(0, 1, "Mode: POLLING (interrupts failed)", 0x0E);
    }
    
    vga_write(0, 2, "Type 'help' for commands", 0x07);
    vga_write(0, 3, "> ", 0x0F);
    
    let mut shell = Shell::new();
    let mut current_row: usize = 3;
    let use_interrupts = final_kb > 0;
    
    loop {
        let scancode = if use_interrupts {
            interrupts::read_scancode()
        } else {
            poll_keyboard()
        };
        
        if let Some(sc) = scancode {
            if let Some(ch) = scancode_to_char(sc) {
                handle_char(&mut shell, &mut current_row, ch);
            }
        }
        
        x86_64::instructions::hlt();
    }
}

fn poll_keyboard() -> Option<u8> {
    use x86_64::instructions::port::Port;
    let mut status_port = Port::<u8>::new(0x64);
    let status = unsafe { status_port.read() };
    
    if status & 0x01 != 0 {
        let mut data_port = Port::<u8>::new(0x60);
        Some(unsafe { data_port.read() })
    } else {
        None
    }
}

fn handle_char(shell: &mut Shell, current_row: &mut usize, ch: char) {
    if ch == '\n' {
        let result = shell.execute();
        
        *current_row += 1;
        if *current_row >= 24 {
            scroll_up();
            *current_row = 23;
        }
        
        match result {
            ShellResult::Clear => {
                clear_screen();
                vga_write(0, 0, "=== AerogelOS Shell ===", 0x0E);
                vga_write(0, 1, "Type 'help' for commands", 0x07);
                *current_row = 2;
            },
            ShellResult::Exit => {
                vga_write(0, *current_row, "Shutting down...", 0x0C);
                use x86_64::instructions::port::Port;
                let mut port = Port::<u16>::new(0x604);
                unsafe { port.write(0x2000); }
                loop { x86_64::instructions::hlt(); }
            },
            ShellResult::MemTest => {
                use alloc::vec::Vec;
                use alloc::string::String;
                
                let mut test_vec = Vec::new();
                for i in 0..10 {
                    test_vec.push(i);
                }
                let _test_string = String::from("Heap works!");
                
                vga_write(0, *current_row, "Vec: OK, String: OK", 0x0A);
                *current_row += 1;
                if *current_row >= 24 {
                    scroll_up();
                    *current_row = 23;
                }
            },
            ShellResult::Output(text) => {
                vga_write(0, *current_row, text, 0x0A);
                *current_row += 1;
                if *current_row >= 24 {
                    scroll_up();
                    *current_row = 23;
                }
            },
            ShellResult::Echo(buf, len) => {
                let text = core::str::from_utf8(&buf[..len]).unwrap_or("");
                vga_write(0, *current_row, text, 0x0A);
                *current_row += 1;
                if *current_row >= 24 {
                    scroll_up();
                    *current_row = 23;
                }
            },
            ShellResult::Empty => {},
        }
        
        vga_write(0, *current_row, "> ", 0x0F);
        
    } else if ch == '\x08' {
        shell.backspace();
        clear_line(*current_row);
        vga_write(0, *current_row, "> ", 0x0F);
        vga_write(2, *current_row, shell.get_buffer(), 0x0F);
        
    } else {
        shell.add_char(ch);
        vga_write(2, *current_row, shell.get_buffer(), 0x0F);
    }
}

fn write_msg(buf: &mut [u8], msg: &[u8]) {
    let len = msg.len().min(buf.len());
    buf[..len].copy_from_slice(&msg[..len]);
}

fn write_u64_at(buf: &mut [u8], offset: usize, mut num: u64) {
    for i in (0..10).rev() {
        if offset + i < buf.len() {
            buf[offset + i] = b'0' + (num % 10) as u8;
            num /= 10;
        }
    }
}

fn vga_write(x: usize, y: usize, s: &str, color: u8) {
    let vga = 0xb8000 as *mut u8;
    let offset = (y * 80 + x) * 2;
    for (i, byte) in s.bytes().enumerate() {
        if x + i < 80 {
            unsafe {
                *vga.offset((offset + i * 2) as isize) = byte;
                *vga.offset((offset + i * 2 + 1) as isize) = color;
            }
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
                let src = ((row * 80 + col) * 2) as isize;
                let dst = (((row - 1) * 80 + col) * 2) as isize;
                *vga.offset(dst) = *vga.offset(src);
                *vga.offset(dst + 1) = *vga.offset(src + 1);
            }
        }
        for col in 0..80 {
            *vga.offset(((24 * 80 + col) * 2) as isize) = b' ';
            *vga.offset(((24 * 80 + col) * 2 + 1) as isize) = 0x0F;
        }
    }
}

fn scancode_to_char(scancode: u8) -> Option<char> {
    if scancode & 0x80 != 0 { return None; }
    
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
        0x39 => Some(' '), 0x1C => Some('\n'), 0x0E => Some('\x08'),
        _ => None,
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    vga_write(0, 20, "!!! KERNEL PANIC !!!", 0x4F);
    loop { x86_64::instructions::hlt(); }
}