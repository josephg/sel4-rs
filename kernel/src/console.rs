//! This is a simple wrapper around a serial (UART) controller for printing debug stuff out from
//! the kernel.

use core::fmt;
use core::fmt::Write;
use crate::racycell::RacyCell;

struct DebugConsole(uart_16550::SerialPort);

static DEBUG_PORT: RacyCell<DebugConsole> = RacyCell::new(DebugConsole(unsafe {
    uart_16550::SerialPort::new(0x3F8)
}));


impl fmt::Write for DebugConsole {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for char in s.bytes() {
            match char {
                b'\n' => self.0.write_str("\r\n").unwrap(),
                byte => self.0.send(byte),
            }
        }
        Ok(())
    }
}

pub fn serial_print(args: fmt::Arguments) {
    // let mut port = DEBUG_PORT.lock();
    // let _ = port.write_fmt(args);
    unsafe { DEBUG_PORT.get_mut() }.write_fmt(args).unwrap();
}

/// SAFETY: This should only be called once at startup.
pub(crate) unsafe fn init_serial() {
    unsafe { DEBUG_PORT.get_mut() }.0.init();
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {{
        $crate::console::serial_print(format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! kprintln {
    () => { $crate::console::kprint!("\n") };
    ($($arg:tt)*) => {{
        $crate::console::serial_print(format_args!($($arg)*));
        $crate::console::serial_print(format_args!("\n"));
    }};}

