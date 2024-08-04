use crate::*;
use crate::syscall::*;
use alloc::string::{String, ToString};
use alloc::vec;

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

impl Stdin {
    fn new() -> Self {
        Self
    }

    pub fn read_key(&self) -> char {
        let mut buffer = [0u8; 4];
        let mut len = 0;
        let mut buf:[u8;1] = [0];
        loop {
            let ret = sys_read(0,&mut buf);
            if let Some(l) = ret {
                if l == 0{
                    continue;
                }
            }
            buffer[len] = buf[0];
            len += 1;
    
            if let Ok(s) = core::str::from_utf8(&buffer[..len]) {
                if let Some(ch) = s.chars().next() {
                    return ch;
                }
            }
    
            if len == 4 {
                len = 0; // Reset buffer if full but no valid character decoded
            }
        }
    }

    pub fn read_line(&self) -> core::result::Result<String, (String, char)> {
        // FIXME: allocate string
        // FIXME: read from input buffer
        //       - maybe char by char?
        // FIXME: handle backspace / enter...
        // FIXME: return string
        let err_key = vec![('\x03',"^C"),('\x04',"^D")];
        let mut line = String::with_capacity(128);
        loop {
            let key = self.read_key();
            // println!("{}",key as u32);
            match key {
                '\r' => {
                    println!(); // Print a new line on the screen
                    break;
                }
                '\u{8}' | '\u{7f}' => { // Backspace or Delete
                    if !line.is_empty() {
                        line.pop();
                        print!("\x08 \x08"); // Move the cursor back, print a space, and move back again
                    }
                }
                '\x03' | '\x04' => {
                    let err = err_key.iter().find(|(err, _)| *err == key);
                    println!("{}", err.unwrap().1);
                    return Err((line, key));
                },
                _ => {
                    line.push(key);
                    print!("{}", key); // Print the character on the screen
                }
            }
        }
        Ok(line)
    }
}

impl Stdout {
    fn new() -> Self {
        Self
    }

    pub fn write(&self, s: &str) {
        sys_write(1, s.as_bytes());
    }
}

impl Stderr {
    fn new() -> Self {
        Self
    }

    pub fn write(&self, s: &str) {
        sys_write(2, s.as_bytes());
    }
}

pub fn stdin() -> Stdin {
    Stdin::new()
}

pub fn stdout() -> Stdout {
    Stdout::new()
}

pub fn stderr() -> Stderr {
    Stderr::new()
}
