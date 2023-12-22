#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod vga_buffer;
mod serial;

use vga_buffer::Color;
use core::panic::PanicInfo;

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    text_color!(Color::White, Color::Red);
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(info)
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello!");
    text_color!(Color::Green);
    println!("We've been trying to reach you about...");
    text_color!(Color::Red, Color::Yellow);
    print!("...YOUR CAR'S EXTENDED WARRANTY!");
    text_color!();
    println!();

    #[cfg(test)]
    test_main();

    loop {}
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