//! Memory subsystem — initialises the frame allocator, page-table mapper,
//! and the kernel heap.

pub mod allocator;
pub mod frame_allocator;
pub mod paging;

use bootloader::BootInfo;
use x86_64::VirtAddr;

/// Initialise the full memory subsystem from the bootloader's `BootInfo`.
pub fn init(boot_info: &'static BootInfo) {
    // Physical frame allocator.
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { paging::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { frame_allocator::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // Kernel heap.
    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("kernel heap initialisation failed");

    crate::serial_println!("[memory] frame allocator + heap initialised");
}
