use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;

// 스캔코드 버퍼 (원형 큐)
static mut SCANCODE_BUFFER: [u8; 16] = [0; 16];
static mut BUFFER_HEAD: usize = 0;
static mut BUFFER_TAIL: usize = 0;

// 디버깅용 카운터
static mut TIMER_TICKS: u64 = 0;
static mut KEYBOARD_INTERRUPTS: u64 = 0;

pub fn read_scancode() -> Option<u8> {
    unsafe {
        if BUFFER_HEAD != BUFFER_TAIL {
            let scancode = SCANCODE_BUFFER[BUFFER_HEAD];
            BUFFER_HEAD = (BUFFER_HEAD + 1) % 16;
            Some(scancode)
        } else {
            None
        }
    }
}

pub fn get_timer_ticks() -> u64 {
    unsafe { TIMER_TICKS }
}

pub fn get_keyboard_interrupts() -> u64 {
    unsafe { KEYBOARD_INTERRUPTS }
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
        
        // CPU 예외 핸들러들
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
        
        // 타이머와 키보드 인터럽트 핸들러
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
        // PIC를 완전히 리셋
        let mut pic1_cmd = Port::<u8>::new(0x20);
        let mut pic1_data = Port::<u8>::new(0x21);
        let mut pic2_cmd = Port::<u8>::new(0xA0);
        let mut pic2_data = Port::<u8>::new(0xA1);
        
        // ICW1: 초기화 시작
        pic1_cmd.write(0x11);
        pic2_cmd.write(0x11);
        
        // ICW2: 인터럽트 벡터 오프셋 설정
        pic1_data.write(PIC_1_OFFSET);
        pic2_data.write(PIC_2_OFFSET);
        
        // ICW3: 캐스케이드 설정
        pic1_data.write(0x04); // PIC2가 IRQ2에 연결
        pic2_data.write(0x02); // PIC2의 캐스케이드 ID
        
        // ICW4: 8086 모드
        pic1_data.write(0x01);
        pic2_data.write(0x01);
        
        // 타이머(IRQ0)와 키보드(IRQ1)만 활성화
        pic1_data.write(0b11111100); // IRQ0, IRQ1 활성화
        pic2_data.write(0b11111111); // 모든 IRQ 비활성화
    }
}

pub fn enable_interrupts() {
    x86_64::instructions::interrupts::enable();
}

// 인터럽트 플래그 확인 함수
pub fn are_interrupts_enabled() -> bool {
    use x86_64::registers::rflags::{self, RFlags};
    rflags::read().contains(RFlags::INTERRUPT_FLAG)
}

// 타이머 인터럽트 핸들러
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    
    unsafe {
        TIMER_TICKS += 1;
        // PIC에 직접 EOI 전송
        let mut pic1_cmd = Port::<u8>::new(0x20);
        pic1_cmd.write(0x20);
    }
}


// 키보드 인터럽트 핸들러
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    
    unsafe {
        KEYBOARD_INTERRUPTS += 1;
        
        // 스캔코드 읽기
        let mut port = Port::<u8>::new(0x60);
        let scancode: u8 = port.read();
        
        // 버퍼에 저장
        let next_tail = (BUFFER_TAIL + 1) % 16;
        if next_tail != BUFFER_HEAD {
            SCANCODE_BUFFER[BUFFER_TAIL] = scancode;
            BUFFER_TAIL = next_tail;
        }
        
        // PIC에 직접 EOI 전송
        let mut pic1_cmd = Port::<u8>::new(0x20);
        pic1_cmd.write(0x20);
    }
}

// CPU 예외 핸들러들
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