#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

extern crate alloc;

use core::panic::PanicInfo;

mod allocator;
mod fs;
mod system_monitor;
mod terminal;
mod vga_buffer;

#[global_allocator]
static ALLOCATOR: allocator::LinkedListAllocator = allocator::LinkedListAllocator::new();

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("TerraOS - A minimal OS with real filesystem!");
    println!("Kernel started successfully!");
    
    // 启用终端初始化，传递分配器实例
    terminal::init(&ALLOCATOR);
    
    // This function should not return
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("\n*** KERNEL PANIC ***");
    println!("System halted due to panic");
    loop {}
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    println!("\n*** ALLOCATION ERROR ***");
    println!("Memory allocation failed: {:?}", layout);
    loop {}
}