use x86_64::instructions::port::Port;
use vga_driver::{println, print_colored, Color};

const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;
const PIC_EOI: u8 = 0x20;

const ICW1_INIT: u8 = 0x11;
const ICW4_8086: u8 = 0x01;

pub fn init_pic() {
    unsafe {
        let mut pic1_command = Port::<u8>::new(PIC1_COMMAND);
        let mut pic1_data = Port::<u8>::new(PIC1_DATA);
        let mut pic2_command = Port::<u8>::new(PIC2_COMMAND);
        let mut pic2_data = Port::<u8>::new(PIC2_DATA);

        // PIC 초기화 시작
        pic1_command.write(ICW1_INIT);
        io_wait();
        pic2_command.write(ICW1_INIT);
        io_wait();

        // 벡터 오프셋 설정 (마스터: 32, 슬레이브: 40)
        pic1_data.write(32);
        io_wait();
        pic2_data.write(40);
        io_wait();

        // 캐스케이드 설정
        pic1_data.write(4);  // IRQ 2에 슬레이브 연결
        io_wait();
        pic2_data.write(2);  // 슬레이브 식별자
        io_wait();

        // 8086 모드 설정
        pic1_data.write(ICW4_8086);
        io_wait();
        pic2_data.write(ICW4_8086);
        io_wait();

        // 모든 IRQ 마스크 (나중에 필요한 것만 언마스크)
        pic1_data.write(0xFF);
        pic2_data.write(0xFF);
    }

    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("PIC 초기화 완료 (모든 IRQ 마스크됨)");
}

pub fn end_of_interrupt(vector_number: u8) {
    unsafe {
        let mut pic1_command = Port::<u8>::new(PIC1_COMMAND);
        let mut pic2_command = Port::<u8>::new(PIC2_COMMAND);

        // 슬레이브 PIC 인터럽트 (IRQ 8-15, 벡터 40-47)
        if vector_number >= 40 && vector_number < 48 {
            pic2_command.write(PIC_EOI);
        }

        // 마스터 PIC에는 항상 EOI 전송
        pic1_command.write(PIC_EOI);
    }
}

fn io_wait() {
    unsafe {
        let mut port = Port::<u8>::new(0x80);
        port.write(0);
    }
}