//! GDT + TSS, with a dedicated stack for the double-fault handler. Adapted
//! from the real blog_os reference (os.phil-opp.com/double-fault-exceptions)
//! — a kernel-stack overflow needs a handler that runs on a stack OTHER
//! than the one that just overflowed, or the CPU triple-faults (reboots)
//! instead of running the handler at all.
use lazy_static::lazy_static;
use x86_64::instructions::segmentation::{Segment, CS, SS};
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(&raw const STACK);
            stack_start + STACK_SIZE as u64
        };
        tss
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.append(Descriptor::kernel_code_segment());
        let data_selector = gdt.append(Descriptor::kernel_data_segment());
        let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code_selector, data_selector, tss_selector })
    };
}

pub fn init() {
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_selector);
        // SS must be reloaded too: the bootloader's own SS selector index
        // can collide with wherever this new GDT places its own entries
        // (the TSS descriptor takes 2 slots) — iretq validates SS against
        // the *currently loaded* GDT, so a stale index here double-faults
        // on the very first interrupt return. Confirmed by hand: the
        // breakpoint handler ran and returned correctly, but the very next
        // instruction after it double-faulted, with SS still pointing at
        // an index that had become part of the TSS descriptor.
        SS::set_reg(GDT.1.data_selector);
        load_tss(GDT.1.tss_selector);
    }
}
