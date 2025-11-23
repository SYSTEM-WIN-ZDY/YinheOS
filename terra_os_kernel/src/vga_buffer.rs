use volatile::Volatile;
use core::fmt;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub const fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

// VGA文本缓冲区常量
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

// 双缓冲实现
#[repr(transparent)]
pub struct DoubleBuffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

impl DoubleBuffer {
    pub const fn new() -> Self {
        DoubleBuffer {
            chars: [[ScreenChar {
                ascii_character: b' ',
                color_code: ColorCode::new(Color::Black, Color::Black),
            }; BUFFER_WIDTH]; BUFFER_HEIGHT],
        }
    }

    pub fn clear(&mut self) {
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.chars[row][col] = ScreenChar {
                    ascii_character: b' ',
                    color_code: ColorCode::new(Color::Black, Color::Black),
                };
            }
        }
    }

    pub fn write_char(&mut self, row: usize, col: usize, char: u8, color_code: ColorCode) {
        if row < BUFFER_HEIGHT && col < BUFFER_WIDTH {
            self.chars[row][col] = ScreenChar {
                ascii_character: char,
                color_code,
            };
        }
    }

    pub fn flush_to_vga(&self) {
        let vga_buffer = unsafe { &mut *(0xb8000 as *mut [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT]) };
        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                volatile::Volatile::write(&mut vga_buffer[row][col], self.chars[row][col]);
            }
        }
    }

    pub fn scroll_up(&mut self) {
        // 将所有行向上滚动一行
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.chars[row - 1][col] = self.chars[row][col];
            }
        }

        // 清除最后一行
        for col in 0..BUFFER_WIDTH {
            self.chars[BUFFER_HEIGHT - 1][col] = ScreenChar {
                ascii_character: b' ',
                color_code: ColorCode::new(Color::Black, Color::Black),
            };
        }
    }
}

/// Prints the given formatted string to the VGA text buffer
/// through the global `WRITER` instance.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

/// Prints the given formatted string to the VGA text buffer
/// through the global `WRITER` instance, appending a newline.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use crate::terminal::Terminal;
    let mut terminal = Terminal::new();
    terminal.write_fmt(args).unwrap();
    terminal.flush(); // 确保输出立即显示
}