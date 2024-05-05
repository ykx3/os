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

    pub fn read_line(&self) -> String {
        // FIXME: allocate string
        // FIXME: read from input buffer
        //       - maybe char by char?
        // FIXME: handle backspace / enter...
        // FIXME: return string
        let mut line = String::with_capacity(128);
        // print!("1");
        let mut buf:[u8;1]=[0];
        loop {
            // print!("2");
            let ret = sys_read(0,&mut buf);
            if let Some(l) = ret {
                if l == 0{
                    continue;
                }
            }
            let key = buf[0] as char;
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
                _ => {
                    line.push(key);
                    print!("{}", key); // Print the character on the screen
                }
            }
        }
        line
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
