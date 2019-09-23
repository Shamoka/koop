#![no_std]

use spinlock;

pub static TEXT_BUFFER: spinlock::Mutex<TextBuffer> = spinlock::Mutex::new(TextBuffer {
    addr: 0xb8000 as *mut Buffer,
    row: 0,
    col: 0
});

type Buffer = [[u16; BUFFER_WIDTH]; BUFFER_HEIGHT];

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

pub struct TextBuffer {
    addr: *mut Buffer,
    row: usize,
    col: usize
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Color {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Ref = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    Gray = 0x7,
    DarkGray = 0x8,
    BrightBlue = 0x9,
    BrightGreen = 0xa,
    BrightCyan = 0xb,
    BrightRef = 0xc,
    BrightMagenta = 0xd,
    Yellow = 0xe,
    White = 0xf
}

impl TextBuffer {
    pub fn write(&mut self, s: &str, bg: Color, fg: Color) {
        for byte in s.bytes() {
            match byte {
                0x20..=0xfe => self.write_byte(byte, bg, fg),
                b'\n' => self.newline(),
                _ => ()
            };
        }
    }

    pub fn clear(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                unsafe {
                    (*self.addr)[row][col] = 0 as u16;
                }
            }
        }
    }

    fn write_byte(&mut self, c: u8, bg: Color, fg: Color) {
        let code: u16 = ((fg as u16 ) << 12) | ((bg as u16) << 8) | (c as u16);

        unsafe {
            (*self.addr)[self.row][self.col] = code;
        }
        self.move_cursor();
    }

    fn newline(&mut self) {
        self.col = 0;
        self.row += 1;
        if self.row >= BUFFER_HEIGHT {
            self.row = BUFFER_HEIGHT - 1;
            self.scroll();
        }
    }

    fn move_cursor(&mut self) {
        self.col += 1;
        if self.col >= BUFFER_WIDTH {
            self.newline();
        }
    }

    fn scroll(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                unsafe {
                    (*self.addr)[row - 1][col] = (*self.addr)[row][col];
                }
            }
        }
        unsafe {
            (*self.addr)[self.row] = [0 as u16; BUFFER_WIDTH];
        }
    }
}

impl core::fmt::Write for TextBuffer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s, Color::White, Color::Black);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($args:tt)*) => ($crate::_print(format_args!($($args)*)));
}

#[macro_export]
macro_rules! println {
    () => ($print!("\n"));
    ($($args:tt)*) => ($crate::print!("{}\n", format_args!($($args)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    TEXT_BUFFER.lock().write_fmt(args).unwrap();
}
