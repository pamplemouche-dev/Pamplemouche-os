//! Interrupt Descriptor Table (IDT) and interrupt handlers.
//!
//! # CPU exceptions
//! Handlers are registered for every x86-64 exception vector.
//!
//! # Hardware IRQs (PIC 8259)
//! IRQ0 → timer  : drives the preemptive scheduler tick.
//! IRQ1 → keyboard: feeds keystrokes into the kernel event queue.

use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

use crate::arch::x86_64::gdt::DOUBLE_FAULT_IST_INDEX;

// ── PIC layout ──────────────────────────────────────────────────────────────

/// Primary PIC base vector — CPU exception vectors 0–31 are reserved.
pub const PIC_1_OFFSET: u8 = 32;
/// Secondary PIC base vector.
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

/// Logical interrupt indices.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

// ── IDT ─────────────────────────────────────────────────────────────────────

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        // CPU exception handlers.
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.divide_error.set_handler_fn(divide_error_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.stack_segment_fault.set_handler_fn(stack_segment_fault_handler);
        idt.segment_not_present.set_handler_fn(segment_not_present_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }

        // Hardware IRQ handlers.
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);

        idt
    };
}

/// Load the IDT into the CPU.
pub fn init_idt() {
    IDT.load();
}

// ── CPU exception handlers ───────────────────────────────────────────────────

extern "x86-interrupt" fn breakpoint_handler(frame: InterruptStackFrame) {
    crate::serial_println!("[EXCEPTION] BREAKPOINT\n{:#?}", frame);
}

extern "x86-interrupt" fn divide_error_handler(frame: InterruptStackFrame) {
    panic!("[EXCEPTION] DIVIDE ERROR\n{:#?}", frame);
}

extern "x86-interrupt" fn invalid_opcode_handler(frame: InterruptStackFrame) {
    panic!("[EXCEPTION] INVALID OPCODE\n{:#?}", frame);
}

extern "x86-interrupt" fn general_protection_fault_handler(
    frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "[EXCEPTION] GENERAL PROTECTION FAULT (code={:#x})\n{:#?}",
        error_code, frame
    );
}

extern "x86-interrupt" fn page_fault_handler(
    frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    panic!(
        "[EXCEPTION] PAGE FAULT\n  address: {:?}\n  error:   {:?}\n{:#?}",
        Cr2::read(),
        error_code,
        frame
    );
}

extern "x86-interrupt" fn stack_segment_fault_handler(
    frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "[EXCEPTION] STACK SEGMENT FAULT (code={:#x})\n{:#?}",
        error_code, frame
    );
}

extern "x86-interrupt" fn segment_not_present_handler(
    frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "[EXCEPTION] SEGMENT NOT PRESENT (code={:#x})\n{:#?}",
        error_code, frame
    );
}

extern "x86-interrupt" fn double_fault_handler(
    frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "[EXCEPTION] DOUBLE FAULT (code={:#x})\n{:#?}",
        error_code, frame
    );
}

// ── Hardware IRQ handlers ────────────────────────────────────────────────────

extern "x86-interrupt" fn timer_interrupt_handler(_frame: InterruptStackFrame) {
    // Tick the scheduler — it decides whether to perform a context switch.
    crate::scheduler::tick();

    // SAFETY: we are inside an IRQ handler; EOI must be sent before returning.
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_frame: InterruptStackFrame) {
    use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, layouts};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(
                ScancodeSet1::new(),
                layouts::Us104Key,
                HandleControl::Ignore,
            ));
    }

    let mut kb = KEYBOARD.lock();
    // SAFETY: 0x60 is the PS/2 data port; reading it consumes the scancode.
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    if let Ok(Some(key_event)) = kb.add_byte(scancode) {
        if let Some(key) = kb.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(c) => crate::serial_print!("{}", c),
                DecodedKey::RawKey(k) => crate::serial_print!("{:?}", k),
            }
        }
    }

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
