use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use lazy_static::lazy_static;
use vga_driver::{println, print_colored, Color};
use x86_64::instructions::port::Port;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(0);
        }
        
        // 키보드 인터럽트 직접 등록
        idt[33].set_handler_fn(keyboard_interrupt_handler);
        
        idt
    };
}

pub fn init_idt() {
    IDT.load();
    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("IDT 로드 및 키보드 핸들러 등록 완료");
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

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    
    print_colored("\n!!! PAGE FAULT !!!\n", Color::White, Color::Red);
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    loop {
        x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    print_colored("[K]", Color::Green, Color::Black);
    
    const KEYBOARD_DATA_PORT: u16 = 0x60;
    
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
    
    let mut data_port = Port::new(KEYBOARD_DATA_PORT);
    let scancode: u8 = unsafe { data_port.read() };
    
    if scancode & 0x80 != 0 {
        unsafe {
            let mut pic1 = Port::<u8>::new(0x20);
            pic1.write(0x20);
        }
        return;
    }

    if let Some(Some(character)) = SCANCODE_TO_ASCII.get(scancode as usize) {
        let mut buf = [0u8; 4];
        let s = character.encode_utf8(&mut buf);
        print_colored(s, Color::LightCyan, Color::Black);
    }
    
    unsafe {
        let mut pic1 = Port::<u8>::new(0x20);
        pic1.write(0x20);
    }
}