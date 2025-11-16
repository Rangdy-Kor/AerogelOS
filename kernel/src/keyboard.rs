use x86_64::instructions::port::Port;
use x86_64::structures::idt::InterruptStackFrame;
use vga_driver::{print_colored, Color};
use crate::pic;

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

pub extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: InterruptStackFrame
) {

	print_colored("!", Color::Red, Color::Black);

    let mut port = Port::new(KEYBOARD_DATA_PORT);
    let scancode: u8 = unsafe { port.read() };

    if scancode & 0x80 != 0 {
        pic::end_of_interrupt(1);
        return;
    }

    if let Some(Some(character)) = SCANCODE_TO_ASCII.get(scancode as usize) {
        // char를 배열로 만들어서 출력
        let mut buf = [0u8; 4];
        let s = character.encode_utf8(&mut buf);
        print_colored(s, Color::LightCyan, Color::Black);
    }

    pic::end_of_interrupt(1);
}