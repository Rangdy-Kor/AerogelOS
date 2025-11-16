use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use lazy_static::lazy_static;
use vga_driver::{println, print_colored, Color};
use pic8259::ChainedPics;
use spin::Mutex;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = 40;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Keyboard = PIC_1_OFFSET + 1,
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt[InterruptIndex::Keyboard as usize].set_handler_fn(keyboard_interrupt_handler);
        
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(0);
        }
        idt
    };
}

pub fn init_idt() {
    IDT.load();
    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("IDT 로드 및 키보드 핸들러 등록 완료");
}

pub fn init_pics() {
    unsafe { PICS.lock().initialize() };
    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("PIC 초기화 완료");
}

pub fn enable_interrupts() {
    x86_64::instructions::interrupts::enable();
    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("CPU 인터럽트 활성화 완료");
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    print_colored("\n!!! BREAKPOINT !!!\n", Color::White, Color::Magenta);
    println!("{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    print_colored("\n!!! DOUBLE FAULT !!!\n", Color::White, Color::Red);
    println!("{:#?}", stack_frame);
    loop {
        x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    
    print_colored("[K]", Color::Green, Color::Black);
    
    static SCANCODE_TO_ASCII: [Option<char>; 58] = [
        None, None, Some('1'), Some('2'), Some('3'), Some('4'), Some('5'), 
        Some('6'), Some('7'), Some('8'), Some('9'), Some('0'), Some('-'), 
        Some('='), None, None, Some('q'), Some('w'), Some('e'), Some('r'), 
        Some('t'), Some('y'), Some('u'), Some('i'), Some('o'), Some('p'), 
        Some('['), Some(']'), Some('\n'), None, Some('a'), Some('s'), 
        Some('d'), Some('f'), Some('g'), Some('h'), Some('j'), Some('k'), 
        Some('l'), Some(';'), Some('\''), Some('`'), None, Some('\\'), 
        Some('z'), Some('x'), Some('c'), Some('v'), Some('b'), Some('n'), 
        Some('m'), Some(','), Some('.'), Some('/'), None, Some('*'), 
        None, Some(' '),
    ];
    
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    
    if scancode & 0x80 == 0 {
        if let Some(Some(ch)) = SCANCODE_TO_ASCII.get(scancode as usize) {
            print_colored(&ch.to_string(), Color::LightCyan, Color::Black);
        }
    }
    
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
    }
}