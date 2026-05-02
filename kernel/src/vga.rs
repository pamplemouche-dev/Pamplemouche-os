//! VGA text-mode driver (80×25 colour console).
//!
//! Provides a global `WRITER` and the `print!` / `println!` macros backed by
//! the VGA text buffer at physical address 0xB8000.

use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

/// VGA colour byte (foreground | background << 4).
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
    fn new(fg: Color, bg: Color) -> Self {
        ColorCode((bg as u8) << 4 | (fg as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii: u8,
    color: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

/// Raw VGA text buffer — accesses must be volatile to prevent optimisation.
#[repr(transparent)]
struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

impl Buffer {
    /// Volatile read of a single screen character.
    fn read(&self, row: usize, col: usize) -> ScreenChar {
        unsafe {
            core::ptr::read_volatile(&self.chars[row][col])
        }
    }
    /// Volatile write of a single screen character.
    fn write(&mut self, row: usize, col: usize, c: ScreenChar) {
        unsafe {
            core::ptr::write_volatile(&mut self.chars[row][col], c)
        }
    }
}

/// Stateful writer that maintains the current cursor row/column.
pub struct Writer {
    col: usize,
    color: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.col >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.col;
                let color = self.color;
                self.buffer.write(row, col, ScreenChar { ascii: byte, color });
                self.col += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe), // non-printable → '■'
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let ch = self.buffer.read(row, col);
                self.buffer.write(row - 1, col, ch);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.col = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii: b' ',
            color: self.color,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.write(row, col, blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        col: 0,
        color: ColorCode::new(Color::LightGreen, Color::Black),
        // SAFETY: 0xB8000 is the well-known VGA text buffer physical address.
        // The bootloader maps physical memory starting at the offset provided
        // in BootInfo; by the time we use this the mapping is in place.
        buffer: unsafe { &mut *(0xB8000 as *mut Buffer) },
    });
}

/// Print the kernel banner to the VGA console.
pub fn print_banner() {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        let mut w = WRITER.lock();
        w.write_string("  Pamplemouche-OS  |  micro-kernel  |  x86_64\n");
        w.write_string("  Darwin compat layer: active\n");
    });
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

/// Print to the VGA console without a trailing newline.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

/// Print to the VGA console with a trailing newline.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(concat!($fmt, "\n"), $($arg)*));
}
