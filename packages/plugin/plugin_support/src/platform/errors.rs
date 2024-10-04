use core::panic::PanicInfo;
use crate::println;

#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "panic occurred at {}:{}: {}",
            location.file(),
            location.line(),
            info.message()
        );
    } else {
        println!("panic occurred: {}", info.message());
    }

    core::intrinsics::abort();
}

