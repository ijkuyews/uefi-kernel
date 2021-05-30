use core::alloc::Layout;
use linked_list_allocator::LockedHeap;
use x86_64::structures::paging::PageTableFlags;

pub const HEAP_START: u64 = 0x_4444_4444_0000;
pub const HEAP_SIZE: u64 = 1024 * 1024;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init() {
  super::map_pages(
    HEAP_START,
    HEAP_START + HEAP_SIZE,
    PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
    true,
  )
  .expect("failed to map heap pages");

  unsafe {
    ALLOCATOR.lock().init(HEAP_START as _, HEAP_SIZE as _);
  }

  log::info!("created heap of size {:#x} at {:#x}", HEAP_SIZE, HEAP_START);
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
  panic!(
    "failed to allocate memory of size {:#x} and layout {:#x}",
    layout.size(),
    layout.align()
  );
}
