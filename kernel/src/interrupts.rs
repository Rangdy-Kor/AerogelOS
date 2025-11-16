use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use lazy_static::lazy_static;
use vga_driver::{println, print_colored, Color};
use crate::keyboard;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        
        // 브레이크포인트 인터럽트
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        
        // 키보드 인터럽트 (IRQ 1 = INT 33)
        idt[33].set_handler_fn(keyboard::keyboard_interrupt_handler);
        
        idt
    };
}

pub fn init_idt() {
    IDT.load();
    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("IDT 로드 완료");
}

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame) 
{
    print_colored("\n\n!!! INTERRUPT !!!\n", Color::White, Color::Magenta);
    println!("Exception: Breakpoint");
    println!("Stack Frame: {:#?}", stack_frame);
}