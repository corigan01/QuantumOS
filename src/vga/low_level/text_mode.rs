use volatile::Volatile;
use lazy_static::lazy_static;
use core::fmt;
use spin::Mutex;


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


#[repr(C)]
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
    row_pos: usize
}



impl Writer {
    pub fn vga_write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.vga_new_line(),
            0x7F => {
                self.column_position -= 1;
            },
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.vga_new_line();
                }

                let row = self.row_pos;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });

                self.column_position += 1;
            }
        }
    }

    pub fn vga_clear_line(&mut self, line: usize) {

        let color_code = self.color_code;

        for clear_col in 0..BUFFER_WIDTH {
            self.buffer.chars[line][clear_col].write(ScreenChar {
                ascii_character: b' ',
                color_code
            });
        }
    }

    pub fn vga_new_line(&mut self) {
        self.row_pos += 1;
        self.column_position = 0;

        if self.row_pos > BUFFER_HEIGHT - 1 {
            self.row_pos = BUFFER_HEIGHT - 1;

            for moving_row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    self.buffer.chars[moving_row - 1][col].write(self.buffer.chars[moving_row][col].read());
                }
            }

            self.vga_clear_line(BUFFER_HEIGHT - 1);

            return;
        }


        self.vga_clear_line(BUFFER_HEIGHT - 1);
    }

    pub fn vga_write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.vga_write_byte(byte),
                // not part of printable ASCII range

                _ => self.vga_write_byte(0xfe),
            }

        }
    }



}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.vga_write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe {&mut *(0xB8000 as *mut Buffer)},
        row_pos: 0
    });
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! vga_print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! vga_println {
    () => ($crate::vga_print!("\n"));
    ($($arg:tt)*) => ($crate::vga_print!("{}\n", format_args!($($arg)*)));
}