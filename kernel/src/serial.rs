//! Serial (COM1) output — the only I/O this kernel has right now. Deliberate:
//! a framebuffer needs a bitmap font renderer before it can print text, a
//! serial line needs nothing but the port, and QEMU's `-serial stdio` makes
//! every boot directly, mechanically verifiable from the host terminal.
use core::fmt;

use spin::Mutex;
use uart_16550::{backend::PioBackend, Config, Uart16550};

pub static SERIAL: Mutex<Option<Uart16550<PioBackend>>> = Mutex::new(None);

/// # Safety
/// Must be called exactly once, before any print!/println! use, from the
/// kernel entry point (single-threaded at that point — no concurrent access).
pub unsafe fn init() {
    let mut uart = Uart16550::new_port(0x3f8).expect("COM1 port should exist on x86_64");
    uart.init(Config::default()).expect("serial init should not fail under QEMU");
    *SERIAL.lock() = Some(uart);
}

struct SerialWriter;

impl fmt::Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if let Some(uart) = SERIAL.lock().as_mut() {
            uart.send_bytes_exact(s.as_bytes());
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    let _ = SerialWriter.write_fmt(args);
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::serial::_print(core::format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", core::format_args!($($arg)*)));
}
