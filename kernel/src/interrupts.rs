//! IDT setup. Adapted from the real blog_os reference
//! (os.phil-opp.com/cpu-exceptions, .../double-fault-exceptions) — the
//! breakpoint handler exists to prove the IDT itself works (int3 is the
//! standard way to test this); the double-fault handler exists so a bug
//! crashes with a message on this kernel's own serial line instead of
//! silently triple-faulting (an instant, unexplained QEMU reboot).
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::gdt;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    crate::println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    println!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    loop {
        x86_64::instructions::hlt();
    }
}
