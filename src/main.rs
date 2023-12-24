#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use rust_os::{vga_buffer, hlt_loop};

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use rust_os::text_color;
    use vga_buffer::Color;

    // TODO - update to println_color! macro when it works with bg color
    text_color!(Color::White, Color::Red);
    rust_os::println!("{}", info);
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info)
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    rust_os::init();

    rust_os::println_color!(vga_buffer::Color::Cyan, "Lets goooo!!!!");
    
    #[cfg(test)]
    test_main();

    hlt_loop();
}

// Tests proper output for successful tests
#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

// Tests proper output for failed tests
// #[test_case]
// fn impossible_assertion() {
//     assert_eq!(0, 1);
// }