//! kernel8 — a minimal, real, bootable x86_64 kernel. This is the
//! honest starting point, not a finished OS: it boots, initializes serial,
//! sets up a GDT/IDT with a double-fault handler, and prints. Paging, a
//! heap, a scheduler, syscalls, and a rustybox-linked userland are the
//! actual next milestones (see README), not yet here.
#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

extern crate alloc;

#[macro_use]
mod serial;
mod allocator;
mod credits;
mod gdt;
mod interrupts;
mod memory;
mod task;

use alloc::{boxed::Box, vec::Vec};
use task::{simple_executor::SimpleExecutor, yield_now::yield_now, Task};

use bootloader_api::config::{BootloaderConfig, Mapping};
use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Translate};
use x86_64::VirtAddr;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    unsafe { serial::init() };

    println!("kernel8: booted.");
    credits::print();

    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    println!("gdt + idt loaded.");

    x86_64::instructions::interrupts::int3();
    println!("survived a breakpoint exception. the IDT is real.");

    let phys_mem_offset = VirtAddr::new(
        boot_info.physical_memory_offset.into_option().expect("physical memory not mapped"),
    );
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_regions) };

    // Proof, not a claim: allocate a real frame, map a brand-new virtual
    // page to it, write a known value through that mapping, and read it
    // back through the SAME page (not the physical-memory-offset view) —
    // if paging were wrong, this would fault or read back garbage.
    let page = Page::containing_address(VirtAddr::new(0x1000_0000_0000));
    let frame = frame_allocator.allocate_frame().expect("no usable frames left");
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    unsafe {
        mapper.map_to(page, frame, flags, &mut frame_allocator)
            .expect("map_to failed")
            .flush();
    }
    let ptr = page.start_address().as_mut_ptr::<u64>();
    unsafe { ptr.write_volatile(0xC0FFEE_u64) };
    let read_back = unsafe { ptr.read_volatile() };
    println!("mapped a new page, wrote 0x{:X}, read back 0x{:X}.", 0xC0FFEE_u64, read_back);
    assert_eq!(read_back, 0xC0FFEE, "paging is lying");

    let translated = mapper.translate_addr(page.start_address());
    println!("translate_addr(new page) -> {:?} (should be frame {:?})", translated, frame.start_address());
    println!("paging works.");

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap init failed");

    // Proof: a real Box and a real Vec, on a real kernel heap, growing.
    let boxed = Box::new(1991);
    println!("heap-allocated Box: {}", boxed);

    let mut v = Vec::new();
    for i in 0..500 {
        v.push(i);
    }
    println!("heap-allocated Vec of {} elements, sum = {}", v.len(), v.iter().sum::<i32>());
    println!("the heap works.");

    // Proof: two cooperative tasks, each yielding once mid-execution. If
    // the executor actually interleaves them (not just runs each to
    // completion before starting the next), the output order proves it:
    // a1, b1, a2, b2 — not a1, a2, b1, b2.
    async fn task_a() {
        println!("task a: 1");
        yield_now().await;
        println!("task a: 2");
    }
    async fn task_b() {
        println!("task b: 1");
        yield_now().await;
        println!("task b: 2");
    }
    let mut executor = SimpleExecutor::new();
    executor.spawn(Task::new(task_a()));
    executor.spawn(Task::new(task_b()));
    executor.run();
    println!("cooperative multitasking works (interleaved output above, not sequential).");

    // From here on, real hardware interrupts are live: the timer fires
    // continuously (each tick prints a '.', proof it's really firing, not
    // asserted), and hlt in the loop below is now the actual idle
    // instruction — the CPU sleeps until the next interrupt wakes it,
    // rather than spinning.
    x86_64::instructions::interrupts::enable();
    println!("interrupts enabled. timer ticks below (one '.' per tick):");

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
