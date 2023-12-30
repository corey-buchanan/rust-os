use core::sync::atomic::{AtomicBool, Ordering};
use bootloader::bootinfo::{MemoryRegionType, MemoryMap};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{Page, PhysFrame, Size4KiB, Mapper, OffsetPageTable, PageTable, FrameAllocator};


static INITIALIZED: AtomicBool = AtomicBool::new(false);
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static>{
    if INITIALIZED.swap(true, Ordering::SeqCst) {
        panic!("memory::init() can only be called once!");
    }

    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();

        let usable_regions = regions.filter(
            |r| r.region_type == MemoryRegionType::Usable);
        
        let addr_ranges = usable_regions.map(
            |r| r.range.start_addr()..r.range.end_addr());
        
        let frame_addresses = addr_ranges.flat_map(
            |r| r.step_by(4096));
        
        frame_addresses.map(
            |addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        // TODO: update to use named existential types in our struct (i.e. we don't need to rebuild the usable_Frames iterator every time)
        // https://github.com/rust-lang/rfcs/pull/2071
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
// pub struct EmptyFrameAllocator;

// unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
//     fn allocate_frame(&mut self) -> Option<PhysFrame> {
//         None
//     }
// }

// pub fn create_example_mapping(
//     page: Page,
//     mapper: &mut OffsetPageTable,
//     frame_allocator: &mut impl FrameAllocator<Size4KiB>,
// ) {
//     use x86_64::structures::paging::PageTableFlags as Flags;

//     // FIXME: Delete as this is not safe
//     let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
//     let flags = Flags::PRESENT | Flags::WRITABLE;

//     let map_to_result = unsafe {
//         mapper.map_to(page, frame, flags, frame_allocator)
//     };

//     map_to_result.expect("map_to failed").flush();
// }

// pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
//     translate_inner_addr(addr, physical_memory_offset)
// }

// fn translate_inner_addr(addr: VirtAddr, physical_memory_offset: VirtAddr) -> Option<PhysAddr> {
//     use x86_64::structures::paging::page_table::FrameError;

//     // Read the active level 4 table from the CR3 register
//     let (level_4_table_frame, _) = Cr3::read();

//     let table_indices = [
//         addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()
//     ];

//     let mut frame = level_4_table_frame;

//     for &index in &table_indices {
//         // Get a reference to the page table
//         let virt = physical_memory_offset + frame.start_address().as_u64();
//         let table_ptr: *const PageTable = virt.as_ptr();
//         let table = unsafe {&*table_ptr};

//         // Get a reference to the page
//         let entry = &table[index];

//         // Update the frame
//         frame = match entry.frame() {
//             Ok(frame) => frame,
//             Err(FrameError::FrameNotPresent) => return None,
//             Err(FrameError::HugeFrame) => panic!("Huge pages not supported!")
//         };
//     }

//     Some(frame.start_address() + u64::from(addr.page_offset()))
// }