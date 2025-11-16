use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use lazy_static::lazy_static;
use vga_driver::{println, print_colored, Color};

// ----------------------------------------------------
// 1. IDT 싱글톤 정의 (Mutex 필요 없음, IDT는 한 번만 로드)
// ----------------------------------------------------

lazy_static! {
    // static mut 대신 lazy_static을 사용하여 IDT를 안전하게 정의
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        
        // 3번 인터럽트 (브레이크포인트) 핸들러 설정
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        
        // 여기에 다른 인터럽트(타이머, 키보드 등) 핸들러를 추가할 예정입니다.

        idt
    };
}

// ----------------------------------------------------
// 2. 초기화 함수
// ----------------------------------------------------

pub fn init_idt() {
    // IDT를 프로세서에 로드합니다.
    IDT.load();
    
    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("IDT 로드 완료");
}


// ----------------------------------------------------
// 3. 인터럽트 핸들러 함수
// ----------------------------------------------------

// Rust에서 인터럽트 핸들러를 정의할 때는 특별한 ABI(Application Binary Interface)가 필요합니다.
// `extern "x86-interrupt"`는 컴파일러에게 이 함수를 인터럽트 핸들러로 사용하도록 지시합니다.
extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame) 
{
    print_colored("\n\n!!! INTERRUPT !!!\n", Color::White, Color::Magenta);
    println!("Exception: Breakpoint");
    println!("Stack Frame: {:#?}", stack_frame);
}