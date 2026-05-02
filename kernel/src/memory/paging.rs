//! Virtual memory / page-table management.
//!
//! Provides a thin wrapper around the `x86_64` crate's `OffsetPageTable` so
//! the rest of the kernel can map/unmap pages without exposing the raw unsafe
//! interface.

use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags, PhysFrame, Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};

/// Initialise an `OffsetPageTable` from the identity-mapped physical memory
/// provided by the bootloader.
///
/// # Safety
/// `physical_memory_offset` must be the exact offset at which physical memory
/// is linearly mapped into the virtual address space by the bootloader.
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// Map a single 4 KiB page to the given physical frame with the supplied flags.
pub fn map_page(
    page: Page<Size4KiB>,
    frame: PhysFrame,
    flags: PageTableFlags,
    mapper: &mut OffsetPageTable,
    allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), &'static str> {
    use x86_64::structures::paging::mapper::MapToError;
    unsafe {
        mapper
            .map_to(page, frame, flags, allocator)
            .map_err(|e| match e {
                MapToError::FrameAllocationFailed => "frame allocation failed",
                MapToError::ParentEntryHugePage => "parent entry is a huge page",
                MapToError::PageAlreadyMapped(_) => "page already mapped",
            })?
            .flush();
    }
    Ok(())
}

/// Translate a virtual address to its mapped physical address (if any).
pub fn translate(addr: VirtAddr, mapper: &OffsetPageTable) -> Option<PhysAddr> {
    use x86_64::structures::paging::mapper::TranslateResult;
    use x86_64::structures::paging::Translate;
    match mapper.translate(addr) {
        TranslateResult::Mapped { frame, offset, .. } => {
            Some(frame.start_address() + offset)
        }
        _ => None,
    }
}

// ── Internal helpers ─────────────────────────────────────────────────────────

unsafe fn active_level_4_table(
    physical_memory_offset: VirtAddr,
) -> &'static mut x86_64::structures::paging::PageTable {
    use x86_64::registers::control::Cr3;
    let (level_4_frame, _) = Cr3::read();
    let phys = level_4_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let table_ptr: *mut x86_64::structures::paging::PageTable = virt.as_mut_ptr();
    &mut *table_ptr
}
