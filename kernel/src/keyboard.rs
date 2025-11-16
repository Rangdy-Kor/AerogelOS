use x86_64::instructions::port::Port;
use x86_64::structures::idt::InterruptStackFrame;
use vga_driver::{print_colored, Color};
use crate::pic;

const KEYBOARD_DATA_PORT: u16 = 0x60;
const KEYBOARD_STATUS_PORT: u16 = 0x64;
const STATUS_OUTPUT_FULL: u8 = 0x01;

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

// src/keyboard.rs 파일 (수정된 keyboard_interrupt_handler)
pub extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: InterruptStackFrame
) {
    let mut data_port = Port::new(KEYBOARD_DATA_PORT);
    // 상태 포트 확인 로직은 타이밍 문제 해결을 위해 임시로 제거합니다.
    
    // 1. 스캔 코드를 즉시 읽어 키보드 컨트롤러 버퍼를 비웁니다.
    let scancode: u8 = unsafe { data_port.read() };

    // ⭐ 2. PIC에게 EOI 신호를 즉시 보냅니다. (가장 중요) ⭐
    //     스캔 코드 분석이나 출력보다 우선하여 다음 인터럽트 준비를 알립니다.
    //     키보드 인터럽트의 벡터 번호는 33입니다.
    crate::pic::end_of_interrupt(33); 

    // 3. 키 떼기 (Break code)는 여기서 처리 (EOI를 이미 보냈으므로 추가 EOI 불필요)
    if scancode & 0x80 != 0 {
        return;
    }

    // 4. 스캔 코드를 분석하고 출력합니다. (안전하게 수행 가능)
    if let Some(Some(character)) = SCANCODE_TO_ASCII.get(scancode as usize) {
        let mut buf = [0u8; 4];
        let s = character.encode_utf8(&mut buf);
        print_colored(s, Color::LightCyan, Color::Black);
    } 
    // 매핑되지 않은 스캔 코드 (예: Ctrl, Alt)는 아무것도 하지 않고 종료됩니다.
    
    // 함수 종료
}