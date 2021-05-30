mod frame_allocator;
mod heap;

use crate::utils::locked::Locked;

use bootloader::boot_info::MemoryRegions;
use frame_allocator::GlobalFrameAllocator;
use spin::Once;
use x86_64::{
  structures::paging::{mapper::MapToError, FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB},
  PhysAddr, VirtAddr,
};

pub static FRAME_ALLOC: Once<Locked<GlobalFrameAllocator>> = Once::new();
pub static MAPPER: Once<Locked<OffsetPageTable>> = Once::new();

fn active_l4_table(phys_mem_offset: VirtAddr) -> &'static mut PageTable {
  use x86_64::registers::control::Cr3;

  let (l4_table_frame, _) = Cr3::read();

  let phys = l4_table_frame.start_address();
  let virt = phys_mem_offset + phys.as_u64();
  let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

  unsafe { &mut *page_table_ptr }
}

pub fn map_pages(start_addr: u64, end_addr: u64, page_flags: PageTableFlags, inclusive: bool) -> Result<(), MapToError<Size4KiB>> {
  let start_addr = VirtAddr::new(start_addr as u64);
  let end_addr = VirtAddr::new(end_addr as u64);

  let start_page = Page::containing_address(start_addr);
  let end_page = Page::containing_address(end_addr);

  let mut mapper = MAPPER.get().expect("mapper has not been initialized").lock();
  let mut frame_alloc = FRAME_ALLOC.get().expect("frame allocator has not been initialized").lock();

  for page in Page::range(start_page, if inclusive { end_page + 1 } else { end_page }) {
    let frame = frame_alloc.allocate_frame().ok_or(MapToError::FrameAllocationFailed)?;

    unsafe {
      mapper.map_to(page, frame, page_flags, &mut *frame_alloc)?.flush();
    }

    log::trace!(
      "mapped virtual page {:#x} to physical frame {:#x}",
      page.start_address().as_u64(),
      frame.start_address().as_u64()
    );
  }

  Ok(())
}

pub fn identity_map_pages(start_addr: u64, end_addr: u64, page_flags: PageTableFlags, inclusive: bool) -> Result<(), MapToError<Size4KiB>> {
  let start_addr = VirtAddr::new(start_addr as u64);
  let end_addr = VirtAddr::new(end_addr as u64);

  let start_page = Page::containing_address(start_addr);
  let end_page = Page::containing_address(end_addr);

  let mut mapper = MAPPER.get().expect("mapper has not been initialized").lock();
  let mut frame_alloc = FRAME_ALLOC.get().expect("frame allocator has not been initialized").lock();

  for page in Page::range(start_page, if inclusive { end_page + 1 } else { end_page }) {
    let frame = PhysFrame::containing_address(PhysAddr::new(page.start_address().as_u64()));

    unsafe {
      mapper.map_to(page, frame, page_flags, &mut *frame_alloc)?.flush();
    }

    log::trace!(
      "mapped virtual page {:#x} to physical frame {:#x}",
      page.start_address().as_u64(),
      frame.start_address().as_u64()
    );
  }

  Ok(())
}

pub fn init(phys_mem_offset: u64, mem_regions: &'static MemoryRegions) {
  let phys_mem_offset = VirtAddr::new(phys_mem_offset);

  unsafe {
    FRAME_ALLOC.call_once(|| Locked::new(GlobalFrameAllocator::new(mem_regions)));
    MAPPER.call_once(|| {
      let l4_table = active_l4_table(phys_mem_offset);

      Locked::new(OffsetPageTable::new(l4_table, phys_mem_offset))
    });
  }

  heap::init();
}
