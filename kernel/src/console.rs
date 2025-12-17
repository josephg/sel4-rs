//! This is a simple wrapper around a serial (UART) controller for printing debug stuff out from
//! the kernel.
//!
//! I'm using ufmt here because its much smaller than core::fmt. core::fmt adds about 60kb to the
//! binary, whereas ufmt only adds about 4.5kb.

use crate::racycell::RacyCell;
use core::convert::Infallible;
use core::fmt::Write;
use ufmt::uWrite;

pub(crate) struct DebugConsole(uart_16550::SerialPort);

pub(crate) static DEBUG_PORT: RacyCell<DebugConsole> = RacyCell::new(DebugConsole(unsafe {
    uart_16550::SerialPort::new(0x3F8)
}));

/// SAFETY: This should only be called once at startup.
pub(crate) unsafe fn init_serial() {
    unsafe { DEBUG_PORT.get_mut() }.0.init();
}

// ufmt, which is like core::fmt but way smaller and faster.
impl uWrite for DebugConsole {
    type Error = Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        for byte in s.bytes() {
            self.0.send(byte);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {{
        let port = unsafe { $crate::console::DEBUG_PORT.get_mut() };
        ufmt::uwrite!(port, $($arg)*);
    }};
}

#[macro_export]
macro_rules! kprintln {
    () => { $crate::console::kprint!("\n") };
    ($($arg:tt)*) => {{
        let port = unsafe { $crate::console::DEBUG_PORT.get_mut() };
        ufmt::uwriteln!(port, $($arg)*);
    }};
}


/// This is a variant of println for printing out warnings.
///
/// Right now this is identical to kprintln, but that may change.
#[macro_export]
macro_rules! kwarnln {
    () => { $crate::console::kprint!("\n") };
    ($($arg:tt)*) => {{
        let port = unsafe { $crate::console::DEBUG_PORT.get_mut() };
        ufmt::uwriteln!(port, $($arg)*);
    }};}

/// Print a message and halt the computer. panic!() will also work, but this adds much less binary
/// size thanks to uDebug.
#[macro_export]
macro_rules! kpanic {
    () => { $crate::console::kprint!("\n") };
    ($($arg:tt)*) => {{
        let port = unsafe { $crate::console::DEBUG_PORT.get_mut() };
        ufmt::uwriteln!(port, $($arg)*);
        $crate::utils::halt();
    }};}


// It'd be nice to skip implementing fmt::Write entirely, because adding this code brings in all
// of rust's formatting infrastructure. Unfortunately, there's no current way to print out panic
// messages without using core::fmt::Arguments and everything that comes along with it.
//
// Maybe at some point I'll put all this stuff behind a feature flag, so you can build the kernel
// into a tiny binary but with worse panic messages. Sadly I don't think most people care.
impl core::fmt::Write for DebugConsole {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            self.0.send(byte);
        }
        Ok(())
    }
}

pub fn serial_print(args: core::fmt::Arguments) {
    // let mut port = DEBUG_PORT.lock();
    // let _ = port.write_fmt(args);
    let port = unsafe { DEBUG_PORT.get_mut() };
    port.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! kprint_big {
    ($($arg:tt)*) => {{
        $crate::console::serial_print(format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! kprintln_big {
    () => { $crate::console::kprint!("\n") };
    ($($arg:tt)*) => {{
        $crate::console::serial_print(format_args!($($arg)*));
        $crate::console::serial_print(format_args!("\n"));
    }};}
