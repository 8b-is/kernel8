//! vakedkernel — a minimal, real, bootable x86_64 kernel. This is the
//! honest starting point, not a finished OS: it boots, initializes serial,
//! sets up a GDT/IDT with a double-fault handler, and prints. Paging, a
//! heap, a scheduler, syscalls, and a rustybox-linked userland are the
//! actual next milestones (see README), not yet here.
#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

#[macro_use]
mod serial;
mod gdt;
mod interrupts;

use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    unsafe { serial::init() };

    println!("vakedkernel: booted.");
    println!("physical_memory_offset: {:?}", boot_info.physical_memory_offset);

    gdt::init();
    interrupts::init_idt();
    println!("gdt + idt loaded.");

    // Proof, not a claim: actually trigger the exception and let the real
    // handler (interrupts.rs) catch it, instead of asserting the IDT works.
    x86_64::instructions::interrupts::int3();
    println!("survived a breakpoint exception. the IDT is real.");

    loop {
        x86_64_hlt();
    }
}

#[inline(always)]
fn x86_64_hlt() {
    unsafe { core::arch::asm!("hlt") };
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("KERNEL PANIC: {}", info);
    loop {
        x86_64_hlt();
    }
}
