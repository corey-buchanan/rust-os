#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod vga_buffer;
mod serial;

use vga_buffer::Color;
use core::panic::PanicInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

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
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {

    #[cfg(test)]
    test_main();

    println!("Hello!");
    text_color!(Color::Green);
    println!("We've been trying to reach you about...");
    text_color!(Color::Red, Color::Yellow);
    print!("...YOUR CAR'S EXTENDED WARRANTY!");
    text_color!();
    println!();

    loop {}
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T where T : Fn() {
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }

    exit_qemu(QemuExitCode::Success);
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