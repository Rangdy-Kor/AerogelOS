use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use lazy_static::lazy_static;
use vga_driver::{println, print_colored, Color, WRITER};
use pic8259::ChainedPics;
use spin::Mutex;
use core::fmt::Write;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = 40;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard = PIC_1_OFFSET + 1,
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        
        // CPU 예외 핸들러
        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.debug.set_handler_fn(debug_handler);
        idt.non_maskable_interrupt.set_handler_fn(nmi_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.overflow.set_handler_fn(overflow_handler);
        idt.bound_range_exceeded.set_handler_fn(bound_range_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.device_not_available.set_handler_fn(device_not_available_handler);
        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        idt.segment_not_present.set_handler_fn(segment_not_present_handler);
        idt.stack_segment_fault.set_handler_fn(stack_segment_handler);
        idt.general_protection_fault.set_handler_fn(general_protection_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.x87_floating_point.set_handler_fn(x87_fpu_handler);
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        idt.machine_check.set_handler_fn(machine_check_handler);
        idt.simd_floating_point.set_handler_fn(simd_fpu_handler);
        
        // 키보드 인터럽트
        idt[InterruptIndex::Timer as usize].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard as usize].set_handler_fn(keyboard_interrupt_handler);
        
        // Double fault (스택 인덱스 제거)
        idt.double_fault.set_handler_fn(double_fault_handler);
        
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

pub fn init_pics() {
    unsafe { PICS.lock().initialize() };
}

pub fn enable_interrupts() {
    x86_64::instructions::interrupts::enable();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    print_colored("\n!!! BREAKPOINT !!!\n", Color::White, Color::Magenta);
    println!("{:#?}", stack_frame);
}

extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    print_colored("\n!!! DIVIDE ERROR !!!\n", Color::White, Color::Red);
    println!("{:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    println!("DEBUG: {:#?}", stack_frame);
}

extern "x86-interrupt" fn nmi_handler(stack_frame: InterruptStackFrame) {
    print_colored("\n!!! NMI !!!\n", Color::White, Color::Red);
    println!("{:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    print_colored("\n!!! OVERFLOW !!!\n", Color::White, Color::Red);
    println!("{:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn bound_range_handler(stack_frame: InterruptStackFrame) {
    print_colored("\n!!! BOUND RANGE EXCEEDED !!!\n", Color::White, Color::Red);
    println!("{:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    print_colored("\n!!! INVALID OPCODE !!!\n", Color::White, Color::Red);
    println!("{:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    print_colored("\n!!! DEVICE NOT AVAILABLE !!!\n", Color::White, Color::Red);
    println!("{:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn invalid_tss_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    print_colored("\n!!! INVALID TSS !!!\n", Color::White, Color::Red);
    println!("Error Code: {}", error_code);
    println!("{:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn segment_not_present_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    print_colored("\n!!! SEGMENT NOT PRESENT !!!\n", Color::White, Color::Red);
    println!("Error Code: {}", error_code);
    println!("{:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn stack_segment_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    print_colored("\n!!! STACK SEGMENT FAULT !!!\n", Color::White, Color::Red);
    println!("Error Code: {}", error_code);
    println!("{:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn general_protection_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    print_colored("\n!!! GENERAL PROTECTION FAULT !!!\n", Color::White, Color::Red);
    println!("Error Code: {}", error_code);
    println!("{:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: x86_64::structures::idt::PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    
    print_colored("\n!!! PAGE FAULT !!!\n", Color::White, Color::Red);
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn x87_fpu_handler(stack_frame: InterruptStackFrame) {
    print_colored("\n!!! x87 FPU ERROR !!!\n", Color::White, Color::Red);
    println!("{:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn alignment_check_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    print_colored("\n!!! ALIGNMENT CHECK !!!\n", Color::White, Color::Red);
    println!("Error Code: {}", error_code);
    println!("{:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    print_colored("\n!!! MACHINE CHECK !!!\n", Color::White, Color::Red);
    println!("{:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn simd_fpu_handler(stack_frame: InterruptStackFrame) {
    print_colored("\n!!! SIMD FPU ERROR !!!\n", Color::White, Color::Red);
    println!("{:#?}", stack_frame);
    loop { x86_64::instructions::hlt(); }
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
            let mut writer = WRITER.lock();
            writer.set_color(Color::LightCyan, Color::Black);
            write!(writer, "{}", ch).unwrap();
            writer.set_color(Color::Yellow, Color::Black);
        }
    }
    
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer as u8);
    }
}