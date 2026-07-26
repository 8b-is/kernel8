//! IDT setup + the PIC/timer/keyboard hardware interrupt path. Adapted from
//! the real blog_os reference (os.phil-opp.com/cpu-exceptions,
//! .../double-fault-exceptions, .../hardware-interrupts) — the breakpoint
//! handler exists to prove the IDT itself works (int3 is the standard way
//! to test this); the double-fault handler exists so a bug crashes with a
//! message on this kernel's own serial line instead of silently
//! triple-faulting (an instant, unexplained QEMU reboot).
use core::sync::atomic::{AtomicU64, Ordering};
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, PS2Keyboard, ScancodeSet1};
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use x86_64::instructions::port::Port;

use crate::gdt;

/// A PIT tick is a naturally quantized event — one indivisible interrupt,
/// never a fraction of one. Counting them with an integer, rather than
/// estimating elapsed time as a continuous `f64` (the usual OS-timekeeping
/// move, and the one this kernel deliberately does NOT make), is the literal
/// meaning of "quantum": quantus, Latin for "how much," is always a count.
/// See https://pocoo.vaked.dev/posts/2026-07-26-quantum-means-counting.
static TICKS: AtomicU64 = AtomicU64::new(0);

/// The exact count of timer interrupts serviced since boot. Integer, exact,
/// no interpolation — "this many, not somewhere-in-between-this-many."
pub fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

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
}

static KEYBOARD: Mutex<PS2Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
    PS2Keyboard::new(ScancodeSet1::new(), layouts::Us104Key, HandleControl::Ignore),
);

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_interrupt_handler);
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

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    TICKS.fetch_add(1, Ordering::Relaxed);
    crate::print!(".");
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

// Wired identically to the timer path (same PIC, same IDT machinery), but
// unlike the timer this genuinely cannot be verified in this environment —
// QEMU here runs headless (-display none) with no interactive input device
// attached, and this sandbox has no way to send a keypress to a subprocess's
// virtual PS/2 controller. Compiles, follows the real reference exactly,
// untested live. Run it yourself with a display to actually type into it.
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    let mut keyboard = KEYBOARD.lock();
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => crate::print!("{}", character),
                DecodedKey::RawKey(key) => crate::print!("{:?}", key),
            }
        }
    }

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
