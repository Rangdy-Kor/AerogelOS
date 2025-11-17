use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;
use lazy_static::lazy_static;
use spin::Mutex;

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
    SCANCODE_BUFFER.lock().pop()
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
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
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
        
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }
        
        idt[InterruptIndex::Timer as usize].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard as usize].set_handler_fn(keyboard_interrupt_handler);
        
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

pub fn init_pics() {
    use x86_64::instructions::port::Port;
    
    unsafe {
        let mut pic1_cmd = Port::<u8>::new(0x20);
        let mut pic1_data = Port::<u8>::new(0x21);
        let mut pic2_cmd = Port::<u8>::new(0xA0);
        let mut pic2_data = Port::<u8>::new(0xA1);
        
        // 기존 마스크 저장
        let mask1 = pic1_data.read();
        let mask2 = pic2_data.read();
        
        // ICW1: 초기화 시작
        pic1_cmd.write(0x11);
        pic2_cmd.write(0x11);
        
        // ICW2: 인터럽트 벡터 오프셋
        pic1_data.write(PIC_1_OFFSET);
        pic2_data.write(PIC_2_OFFSET);
        
        // ICW3: 마스터/슬레이브 연결
        pic1_data.write(0x04);
        pic2_data.write(0x02);
        
        // ICW4: 8086 모드
        pic1_data.write(0x01);
        pic2_data.write(0x01);
        
        // 마스크 복원 후 타이머/키보드만 활성화
        pic1_data.write(mask1 & 0xFC); // IRQ0, IRQ1 활성화
        pic2_data.write(mask2 | 0xFF); // 모든 IRQ2-15 비활성화
    }
}

pub fn enable_interrupts() {
    x86_64::instructions::interrupts::enable();
}

pub fn are_interrupts_enabled() -> bool {
    use x86_64::registers::rflags::{self, RFlags};
    rflags::read().contains(RFlags::INTERRUPT_FLAG)
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    *TIMER_TICKS.lock() += 1;
    
    unsafe {
        use x86_64::instructions::port::Port;
        let mut pic1 = Port::<u8>::new(0x20);
        pic1.write(0x20); // EOI
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    
    *KEYBOARD_INTERRUPTS.lock() += 1;
    
    let mut port = Port::<u8>::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    
    SCANCODE_BUFFER.lock().push(scancode);
    
    unsafe {
        let mut pic1 = Port::<u8>::new(0x20);
        pic1.write(0x20); // EOI
    }
}

extern "x86-interrupt" fn breakpoint_handler(_stack_frame: InterruptStackFrame) {}
extern "x86-interrupt" fn divide_error_handler(_stack_frame: InterruptStackFrame) {
    loop { x86_64::instructions::hlt(); }
}
extern "x86-interrupt" fn debug_handler(_stack_frame: InterruptStackFrame) {}
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
    loop { x86_64::instructions::hlt(); }
}