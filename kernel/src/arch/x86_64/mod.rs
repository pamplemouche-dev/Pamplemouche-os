//! x86-64 initialisation: GDT, IDT, and PIC.

pub mod gdt;
pub mod interrupts;

/// Initialise all x86-64 hardware abstractions in the correct order:
///
/// 1. GDT — provides segment descriptors required for the IDT and TSS.
/// 2. IDT — registers exception and IRQ handlers.
pub fn init() {
    gdt::init();
    interrupts::init_idt();
    // Initialise PIC 8259 and mask all IRQs except the timer (IRQ0) and
    // keyboard (IRQ1) which we handle explicitly.
    unsafe { interrupts::PICS.lock().initialize() };
    crate::serial_println!("[arch/x86_64] GDT + IDT + PIC initialised");
}
