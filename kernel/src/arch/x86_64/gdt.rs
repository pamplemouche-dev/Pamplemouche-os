//! Global Descriptor Table (GDT) and Task State Segment (TSS).
//!
//! The GDT provides the CPU with segment descriptors for kernel/user code and
//! data.  The TSS holds the kernel stack pointer used when an interrupt fires
//! in user mode so that the interrupt handler runs on a known-good stack.

use lazy_static::lazy_static;
use x86_64::instructions::segmentation::{CS, Segment};
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

/// Index of the double-fault interrupt stack in the IST.
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

/// Size of the kernel stacks allocated inside the TSS.
const STACK_SIZE: usize = 4096 * 5;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            // SAFETY: single-core kernel; the stack is a static array.
            let stack_start = VirtAddr::from_ptr(core::ptr::addr_of!(STACK));
            stack_start + STACK_SIZE as u64
        };
        tss
    };
}

/// Segment selectors produced when building the GDT.
pub struct Selectors {
    pub kernel_code: SegmentSelector,
    pub tss: SegmentSelector,
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let kernel_code = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { kernel_code, tss })
    };
}

/// Load the GDT, reload the code segment register, and load the TSS.
pub fn init() {
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.kernel_code);
        load_tss(GDT.1.tss);
    }
}
