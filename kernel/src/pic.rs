use x86_64::instructions::port::Port;
use vga_driver::{println, print_colored, Color};

const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

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

        pic1_data.write(0x00);
        pic2_data.write(0xFF);
    }

    print_colored("[OK] ", Color::LightGreen, Color::Black);
    println!("PIC 초기화 완료");
}

pub fn end_of_interrupt(irq: u8) {
    unsafe {
        let mut pic1_command = Port::<u8>::new(PIC1_COMMAND);
        let mut pic2_command = Port::<u8>::new(PIC2_COMMAND);

        if irq >= 8 {
            pic2_command.write(0x20);
        }
        pic1_command.write(0x20);
    }
}

fn io_wait() {
    unsafe {
        let mut port = Port::<u8>::new(0x80);
        port.write(0);
    }
}