#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use x86_64::structures::paging::Page;
use x86_64::VirtAddr;
use rust_os::{vga_buffer, hlt_loop, memory};

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

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    rust_os::init();

    rust_os::println_color!(vga_buffer::Color::LightCyan, "Lets goooo!!!!");

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    // let page = Page::containing_address(VirtAddr::new(0xdeadbeef));
    // memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    // let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    // unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};
    
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