use core::arch::asm;
use core::panic::PanicInfo;
use crate::kprintln;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!("\n\nKERNEL PANIC! Aaaah!");
    kprintln!("{}", info.message());

    if let Some(location) = info.location() {
        kprintln!("at {:?}", location);
    } else {
        kprintln!("Location unknown");
    }

    unsafe {
        // Stop the machine.
        asm!("hlt");
    }
    loop {}
}