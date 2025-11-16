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
        
        // 키보드 인터럽트만 활성화
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
    unsafe { 
        PICS.lock().initialize();
        // 타이머 비활성화, 키보드만 활성화
        PICS.lock().write_masks(0b11111101, 0b11111111);
    };
}

pub fn enable_interrupts() {
    x86_64::instructions::interrupts::enable();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    let _ = stack_frame; // 사용하지 않음을 명시
}

extern "x86-interrupt" fn divide_error_handler(_stack_frame: InterruptStackFrame) {
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn debug_handler(_stack_frame: InterruptStackFrame) {
}

extern "x86-interrupt" fn nmi_handler(_stack_frame: InterruptStackFrame) {
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn overflow_handler(_stack_frame: InterruptStackFrame) {
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn bound_range_handler(_stack_frame: InterruptStackFrame) {
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn invalid_opcode_handler(_stack_frame: InterruptStackFrame) {
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn device_not_available_handler(_stack_frame: InterruptStackFrame) {
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn invalid_tss_handler(_stack_frame: InterruptStackFrame, _error_code: u64) {
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn segment_not_present_handler(_stack_frame: InterruptStackFrame, _error_code: u64) {
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn stack_segment_handler(_stack_frame: InterruptStackFrame, _error_code: u64) {
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn general_protection_handler(_stack_frame: InterruptStackFrame, _error_code: u64) {
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn page_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: x86_64::structures::idt::PageFaultErrorCode,
) {
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn x87_fpu_handler(_stack_frame: InterruptStackFrame) {
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn alignment_check_handler(_stack_frame: InterruptStackFrame, _error_code: u64) {
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn machine_check_handler(_stack_frame: InterruptStackFrame) -> ! {
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn simd_fpu_handler(_stack_frame: InterruptStackFrame) {
    loop { x86_64::instructions::hlt(); }
}

extern "x86-interrupt" fn double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    
    let mut port = Port::new(0x60);
    let _scancode: u8 = unsafe { port.read() };
    
    // 일단 스캔코드만 읽고 처리는 나중에
    
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer as u8);
    }
}