//! Paging: an `OffsetPageTable` over the bootloader's complete-physical-
//! memory mapping, plus a frame allocator over the usable regions the
//! bootloader reports. Adapted from the real blog_os reference
//! (os.phil-opp.com/paging-implementation), field names updated for the
//! current bootloader_api (`MemoryRegion { start, end, kind }`, not the
//! older `bootloader` crate's `MemoryMap`/range-based shape that guide's
//! text still shows).
use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use x86_64::{
    registers::control::Cr3,
    structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB},
    PhysAddr, VirtAddr,
};

/// # Safety
/// The complete physical memory must actually be mapped at
/// `physical_memory_offset` (true here because the bootloader is configured
/// with `mappings.physical_memory = Some(Mapping::Dynamic)`), and this must
/// be called only once — calling it twice hands out two `&mut` references
/// to the same page table, which is undefined behavior.
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    unsafe { &mut *page_table_ptr }
}

pub struct BootInfoFrameAllocator {
    memory_regions: &'static MemoryRegions,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// # Safety
    /// `memory_regions` must be the real region list the bootloader
    /// reported — this trusts every `Usable` region is actually free.
    pub unsafe fn init(memory_regions: &'static MemoryRegions) -> Self {
        Self { memory_regions, next: 0 }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> + '_ {
        self.memory_regions
            .iter()
            .filter(|r| r.kind == MemoryRegionKind::Usable)
            .flat_map(|r| (r.start..r.end).step_by(4096))
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
