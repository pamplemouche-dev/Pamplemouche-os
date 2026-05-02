//! Kernel heap allocator.
//!
//! The heap lives in a dedicated virtual-address range (`HEAP_START … HEAP_END`)
//! and is backed by a `linked_list_allocator::LockedHeap`.  During
//! initialisation `init_heap` maps physical frames into that range so that
//! `alloc` / `dealloc` work correctly.

use linked_list_allocator::LockedHeap;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

/// Start of the kernel heap in the virtual address space.
pub const HEAP_START: usize = 0x_4444_4444_0000;
/// Heap size: 1 MiB — can be grown later.
pub const HEAP_SIZE: usize = 1024 * 1024;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Map and initialise the kernel heap.
///
/// Must be called exactly once, after the page-table mapper and frame
/// allocator are ready.
pub fn init_heap(
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let start = VirtAddr::new(HEAP_START as u64);
        let end = start + HEAP_SIZE as u64 - 1u64;
        Page::range_inclusive(
            Page::containing_address(start),
            Page::containing_address(end),
        )
    };

    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    // SAFETY: all pages in the range are now mapped and exclusively owned by
    // the heap; the size and base are correct.
    unsafe {
        ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE);
    }

    Ok(())
}
