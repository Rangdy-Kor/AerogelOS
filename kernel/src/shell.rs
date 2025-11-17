// kernel/src/shell.rs - 확장된 버전

pub struct Shell {
    buffer: [u8; 256],
    cursor: usize,
    boot_time: u64, // 부팅 시간 저장
}

pub enum ShellResult {
    Output(&'static str),
    MultiOutput([&'static str; 16], usize), // 여러 줄 출력
    Clear,
    Print([u8; 256], usize),
    Shutdown,
    Reboot,
    CpuInfo,
    MemInfo,
    SysInfo,
    DateTime,
    Uptime(u64), // 현재 틱 전달
    BgColor(u8), // 배경색 코드
    Empty,
}

impl Shell {
    pub const fn new() -> Self {
        Shell {
            buffer: [0; 256],
            cursor: 0,
            boot_time: 0,
        }
    }
    
    pub fn set_boot_time(&mut self, ticks: u64) {
        self.boot_time = ticks;
    }
    
    pub fn add_char(&mut self, ch: char) {
        if self.cursor < 255 {
            self.buffer[self.cursor] = ch as u8;
            self.cursor += 1;
        }
    }
    
    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.buffer[self.cursor] = 0;
        }
    }
    
    pub fn execute(&mut self, current_ticks: u64) -> ShellResult {
        let cmd = core::str::from_utf8(&self.buffer[..self.cursor])
            .unwrap_or("");
        
        let parts: [&str; 8] = {
            let mut p = [""; 8];
            for (i, part) in cmd.trim().split_whitespace().enumerate() {
                if i >= 8 { break; }
                p[i] = part;
            }
            p
        };
        
        let result = match parts[0] {
            "help" => {
                let lines = [
                    "Available commands:",
                    "  help      - Show this message",
                    "  clear     - Clear screen",
                    "  memtest   - Test memory allocator",
                    "  print     - Print text",
                    "  version   - Show OS version",
                    "  shutdown  - Shutdown (QEMU only)",
                    "  reboot    - Reboot system",
                    "  date      - Show current date",
                    "  time      - Show current time",
                    "  uptime    - Show uptime",
                    "  bgcolor   - Change background color (0-F)",
                    "  cpuinfo   - Show CPU information",
                    "  meminfo   - Show memory usage",
                    "  sysinfo   - Show system information",
                    ""
                ];
                ShellResult::MultiOutput(lines, 15)
            },
            "clear" => ShellResult::Clear,
            "memtest" => ShellResult::MemInfo,
            "version" => ShellResult::Output("AerogelOS v0.1.0 - Polling Mode"),
            "shutdown" => ShellResult::Shutdown,
            "reboot" => ShellResult::Reboot,
            "date" => ShellResult::DateTime,
            "time" => ShellResult::DateTime,
            "uptime" => ShellResult::Uptime(current_ticks - self.boot_time),
            "cpuinfo" => ShellResult::CpuInfo,
            "meminfo" => ShellResult::MemInfo,
            "sysinfo" => ShellResult::SysInfo,
            "bgcolor" if parts[1].len() > 0 => {
                // 16진수 파싱 (0-F)
                let color = match parts[1].chars().next() {
                    Some('0') => 0x0,
                    Some('1') => 0x1,
                    Some('2') => 0x2,
                    Some('3') => 0x3,
                    Some('4') => 0x4,
                    Some('5') => 0x5,
                    Some('6') => 0x6,
                    Some('7') => 0x7,
                    Some('8') => 0x8,
                    Some('9') => 0x9,
                    Some('a') | Some('A') => 0xA,
                    Some('b') | Some('B') => 0xB,
                    Some('c') | Some('C') => 0xC,
                    Some('d') | Some('D') => 0xD,
                    Some('e') | Some('E') => 0xE,
                    Some('f') | Some('F') => 0xF,
                    _ => return ShellResult::Output("Invalid color! Use 0-F"),
                };
                ShellResult::BgColor(color)
            },
            "print" if parts[1].len() > 0 => {
                // "print " 이후의 모든 텍스트를 합침
                if let Some(text_start) = cmd.find("print ") {
                    let start_pos = text_start + 6; // "print " 길이
                    let text = if start_pos < cmd.len() {
                        &cmd[start_pos..]
                    } else {
                        ""
                    };
                    let mut print_buf = [0u8; 256];
                    let len = text.len().min(255);
                    if len > 0 {
                        print_buf[..len].copy_from_slice(&text.as_bytes()[..len]);
                        ShellResult::Print(print_buf, len)
                    } else {
                        ShellResult::Output("Usage: print <text>")
                    }
                } else {
                    ShellResult::Output("Usage: print <text>")
                }
            },
            "print" => ShellResult::Output("Usage: print <text>"),
            "" => ShellResult::Empty,
            _ => ShellResult::Output("Unknown command. Type 'help' for commands."),
        };
        
        self.clear();
        result
    }
    
    pub fn clear(&mut self) {
        self.buffer = [0; 256];
        self.cursor = 0;
    }
    
    pub fn get_buffer(&self) -> &str {
        core::str::from_utf8(&self.buffer[..self.cursor])
            .unwrap_or("")
    }
}