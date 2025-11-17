// kernel/src/interrupts.rs

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::instructions::interrupts;
use x86_64::instructions::hlt;

pub struct ScancodeBuffer {
    buffer: [u8; 16],
    head: usize,
    tail: usize,
}

impl ScancodeBuffer {
    const fn new() -> Self {
        ScancodeBuffer {
            buffer: [0; 16],
            head: 0,
            tail: 0,
        }
    }

    pub fn push(&mut self, scancode: u8) -> bool {
        let next_tail = (self.tail + 1) % 16;
        if next_tail != self.head {
            self.buffer[self.tail] = scancode;
            self.tail = next_tail;
            true
        } else {
            false
        }
    }

    pub fn pop(&mut self) -> Option<u8> {
        if self.head != self.tail {
            let scancode = self.buffer[self.head];
            self.head = (self.head + 1) % 16;
            Some(scancode)
        } else {
            None
        }
    }
}

static SCANCODE_BUFFER: Mutex<ScancodeBuffer> = Mutex::new(ScancodeBuffer::new());
static TIMER_TICKS: Mutex<u64> = Mutex::new(0);
static KEYBOARD_INTERRUPTS: Mutex<u64> = Mutex::new(0);

pub fn read_scancode() -> Option<u8> {
    interrupts::without_interrupts(|| {
        SCANCODE_BUFFER.lock().pop()
    })
}

pub fn get_timer_ticks() -> u64 {
    *TIMER_TICKS.lock()
}

pub fn get_keyboard_interrupts() -> u64 {
    *KEYBOARD_INTERRUPTS.lock()
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = 40;
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard = PIC_1_OFFSET + 1,
}

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { core::ptr::addr_of!(STACK) });
            stack_start + STACK_SIZE
        };
        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code_selector, tss_selector })
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init_gdt() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, Segment};
    
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        
        // CPU 예외 핸들러 - 간단한 버전만
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }
        
        // 하드웨어 인터럽트만 등록
        idt[InterruptIndex::Timer as usize].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard as usize].set_handler_fn(keyboard_interrupt_handler);
        
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

pub fn init_pics() {
    unsafe {
        PICS.lock().initialize();
        PICS.lock().write_masks(0b11111100, 0b11111111);
    }
}

pub fn enable_interrupts() {
    x86_64::instructions::interrupts::enable();
}

pub fn are_interrupts_enabled() -> bool {
    use x86_64::registers::rflags::{self, RFlags};
    rflags::read().contains(RFlags::INTERRUPT_FLAG)
}

// 타이머 인터럽트 핸들러
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    *TIMER_TICKS.lock() += 1;
    
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer as u8);
    }
}

// 키보드 인터럽트 핸들러
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    
    *KEYBOARD_INTERRUPTS.lock() += 1;
    
    let mut port = Port::<u8>::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    
    SCANCODE_BUFFER.lock().push(scancode);
    
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
    }
}

// 브레이크포인트 핸들러
extern "x86-interrupt" fn breakpoint_handler(_stack_frame: InterruptStackFrame) {
    // 아무것도 하지 않음
}

// 더블 폴트 핸들러
extern "x86-interrupt" fn double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    // VGA에 직접 에러 메시지 출력
    let vga = 0xb8000 as *mut u16;
    let msg = b"DOUBLE FAULT!";
    for (i, &byte) in msg.iter().enumerate() {
        unsafe {
            *vga.add(i) = (byte as u16) | (0x4F << 8); // 빨간 배경
        }
    }
    loop {
        hlt();
    }
}