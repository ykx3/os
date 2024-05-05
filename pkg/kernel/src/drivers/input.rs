use crossbeam_queue::ArrayQueue;
use lazy_static::lazy_static;
use alloc::string::String;

type Key = char; // 将 Key 类型从 u8 更改为 char

lazy_static! {
    static ref INPUT_BUF: ArrayQueue<Key> = ArrayQueue::new(128);
}

#[inline]
pub fn push_key(key: Key) {
    if INPUT_BUF.push(key).is_err() {
        warn!("Input buffer is full. Dropping key '{:?}'", key);
    }
}

#[inline]
pub fn try_pop_key() -> Option<Key> {
    INPUT_BUF.pop()
}

pub fn pop_key() -> Key {
    loop {
        if let Some(key) = try_pop_key() {
            return key;
        }
    }
}

pub fn get_line() -> String {
    let mut line = String::with_capacity(128);
    loop {
        let key = pop_key();
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
