// kernel/src/shell.rs

pub struct Shell {
    buffer: [u8; 256],
    cursor: usize,
}

pub enum ShellResult {
    Output(&'static str),
    Clear,
    Echo([u8; 256], usize), // buffer와 길이
    Empty,
}

impl Shell {
    pub fn new() -> Self {
        Shell {
            buffer: [0; 256],
            cursor: 0,
        }
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
    
    pub fn execute(&mut self) -> ShellResult {
        let cmd = core::str::from_utf8(&self.buffer[..self.cursor])
            .unwrap_or("");
        
        let result = match cmd.trim() {
            "help" => ShellResult::Output("Commands: help, clear, echo, version"),
            "clear" => ShellResult::Clear,
            "version" => ShellResult::Output("AerogelOS v0.1.0"),
            cmd if cmd.starts_with("echo ") => {
                let text = &cmd[5..];
                let mut echo_buf = [0u8; 256];
                let len = text.len().min(255);
                echo_buf[..len].copy_from_slice(&text.as_bytes()[..len]);
                ShellResult::Echo(echo_buf, len)
            },
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