use volatile::Volatile;
use x86_64::instructions::interrupts;
use core::fmt;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

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
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.new_line();
            }

            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                });
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                // Copy the character from the current row
                let character = self.buffer.chars[row][col].read();
                // Paste the character on the row above
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    fn backspace(&mut self) {
        if self.column_position > 0 {
            let blank = ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            };

            self.column_position -= 1;

            let row = BUFFER_HEIGHT - 1;
            let col = self.column_position;
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! backspace {
    () => ($crate::vga_buffer::_backspace());
}

#[doc(hidden)]
pub fn _backspace() {
    interrupts::without_interrupts(|| {
        WRITER.lock().backspace();
    })
}

#[macro_export]
macro_rules! print_color {
    ($fg:expr, $($arg:tt)*) => ($crate::vga_buffer::_print_color($fg, $crate::vga_buffer::Color::Black, format_args!($($arg)*)));
    // TODO - fix, possibly consolidate with print! macro
    // ($fg:expr, $bg:expr, $($arg:tt)*) => ($crate::vga_buffer::_print_color($fg, $bg, format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println_color {
    ($fg:expr, $($arg:tt)*) => ($crate::print_color!($fg, "{}\n", format_args!($($arg)*)));
    // TODO - fix, possibly consolidate with println! macro
    // ($fg:expr, $bg:expr, $($arg:tt)*) => ($crate::print_color!($fg, $bg, "{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    })
}

#[doc(hidden)]
pub fn _print_color(foreground: Color, background: Color, args: fmt::Arguments) {
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();

        let restore_color = writer.color_code;

        writer.color_code = ColorCode::new(foreground, background);
        writer.write_fmt(args).unwrap();
        writer.color_code = restore_color;
    })
}

#[macro_export]
macro_rules! text_color {
    () => ($crate::vga_buffer::_set_color(Color::White, Color::Black));
    ($fg:expr) => ($crate::vga_buffer::_set_color($fg, Color::Black));
    ($fg:expr, $bg:expr) => ($crate::vga_buffer::_set_color($fg, $bg));
}

#[doc(hidden)]
pub fn _set_color(foreground: Color, background: Color) {
    interrupts::without_interrupts(|| {
        WRITER.lock().color_code = ColorCode::new(foreground, background);
    })
}

#[test_case]
fn test_print_simple() {
    print!("I'm not panicking!");
}

#[test_case]
fn test_print_past_eol() {
    for _ in 0..100 {
        print!("E");
    }
}

#[test_case]
fn test_println_simple() {
    print!("I'm really not panicking!");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("I will not drop a double decker in the toilet...");
    }
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    let s = "Rustoleum";

    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");

        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(c, char::from(screen_char.ascii_character));
        }
    })
}
