use x86_64::instructions::port::Port;
use vga_driver::{println, print_colored, Color};

const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;
const PIC_EOI: u8 = 0x20; // EOI 명령

const ICW1_INIT: u8 = 0x11;
const ICW4_8086: u8 = 0x01;

pub fn init_pic() {
    unsafe {
        let mut pic1_command = Port::<u8>::new(PIC1_COMMAND);
        let mut pic1_data = Port::<u8>::new(PIC1_DATA);
        let mut pic2_command = Port::<u8>::new(PIC2_COMMAND);
        let mut pic2_data = Port::<u8>::new(PIC2_DATA);

        pic1_command.write(ICW1_INIT);
        io_wait();
        pic2_command.write(ICW1_INIT);
        io_wait();

        pic1_data.write(32);
        io_wait();
        pic2_data.write(40);
        io_wait();

        pic1_data.write(4);
        io_wait();
        pic2_data.write(2);
        io_wait();

        pic1_data.write(ICW4_8086);
        io_wait();
        pic2_data.write(ICW4_8086);
        io_wait();

        pic1_data.write(0xFF);
        pic2_data.write(0xFF);
    }

    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("PIC 초기화 완료");
}

pub fn end_of_interrupt(vector_number: u8) { // 인수를 vector_number로 가정
    unsafe {
        let mut pic1_command = Port::<u8>::new(PIC1_COMMAND);
        let mut pic2_command = Port::<u8>::new(PIC2_COMMAND);

        // 인터럽트 벡터 40번(IRQ 8) 이상은 슬레이브 PIC가 처리합니다.
        if vector_number >= 40 {
            // 1. 슬레이브 PIC에게 EOI 신호 전송
            pic2_command.write(PIC_EOI);
            io_wait(); // 명령어 처리 대기 (안정성 강화)
        }

        // 2. 마스터 PIC에게 EOI 신호 전송 (슬레이브 인터럽트의 경우에도 필요)
        pic1_command.write(PIC_EOI);
        io_wait(); // 명령어 처리 대기
    }
}

fn io_wait() {
    unsafe {
        let mut port = Port::<u8>::new(0x80);
        port.write(0);
    }
}