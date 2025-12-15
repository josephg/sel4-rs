use crate::utils::halt;
use crate::{kprintln, kprintln_big};
use core::panic::PanicInfo;

/// NOTE: Its only possible to print out the panic using core::fmt, which adds 60kb or so to the
/// kernel size. I'm going to keep this giant size for now through the panic handler, but in the
/// future I might make the kernel panic free. (Or at least, have a compilation option to avoid
/// pulling in all this crap).
///
/// The nice thing about the rust panic handler is it'll call this path for all out of bounds errors
/// and things like that, which is very useful during development.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!("\n\nKERNEL PANIC! Aaaah!");

    // For small builds, this removes about 20k of formatting infrastructure.
    // let msg = info.message();
    // if let Some(s) = msg.as_str() {
    //     kprintln!("{}", s);
    // }
    kprintln_big!("{}", info.message());

    if let Some(location) = info.location() {
        kprintln_big!("at {:?}", location);
    } else {
        kprintln_big!("Location unknown");
    }

    halt();
}