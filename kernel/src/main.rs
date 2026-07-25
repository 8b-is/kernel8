//! vakedkernel — a minimal, real, bootable x86_64 kernel. This is the
//! honest starting point, not a finished OS: it boots, initializes serial,
//! and prints. GDT/IDT, paging, a scheduler, syscalls, and a rustybox-linked
//! userland are the actual next milestones (see README), not yet here.
#![no_std]
#![no_main]

#[macro_use]
mod serial;

use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    unsafe { serial::init() };

    println!("vakedkernel: booted.");
    println!("physical_memory_offset: {:?}", boot_info.physical_memory_offset);
    println!("this is a real boot, not a simulation of one.");

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
