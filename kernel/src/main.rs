//! Pamplemouche-OS — micro-kernel entry point.
//!
//! The bootloader calls `kernel_main` after setting up the initial page tables
//! and passing a `BootInfo` struct that describes physical memory.

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

mod arch;
mod ipc;
mod memory;
mod scheduler;
mod serial;
mod vga;

entry_point!(kernel_main);

/// Kernel entry point — called by the bootloader.
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    serial_println!("[pamplemouche] boot");

    // 1. Architecture initialisation (GDT, IDT, PIC).
    arch::x86_64::init();

    // 2. Memory subsystem (physical frames → virtual mapping → heap).
    memory::init(boot_info);

    // 3. Scheduler + IPC.
    scheduler::init();
    ipc::init();

    serial_println!("[pamplemouche] kernel ready — enabling interrupts");
    x86_64::instructions::interrupts::enable();

    vga::print_banner();

    // Idle loop — the scheduler will preempt us via the timer IRQ.
    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[PANIC] {}", info);
    loop {
        x86_64::instructions::hlt();
    }
}

#[alloc_error_handler]
fn alloc_error(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout);
}
