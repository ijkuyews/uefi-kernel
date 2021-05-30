use bootloader::boot_info::{MemoryRegionKind, MemoryRegions};
use x86_64::{
  structures::paging::{FrameAllocator, PhysFrame, Size4KiB},
  PhysAddr,
};

pub struct GlobalFrameAllocator {
  mem_maps: &'static MemoryRegions,
  next: usize,
}

impl GlobalFrameAllocator {
  pub fn new(mem_maps: &'static MemoryRegions) -> Self {
    Self { mem_maps, next: 0 }
  }

  fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
    self
      .mem_maps
      .iter()
      .filter(|region| region.kind == MemoryRegionKind::Usable)
      .map(|region| region.start..region.end)
      .flat_map(|range| range.step_by(4096))
      .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
  }
}

unsafe impl Send for GlobalFrameAllocator {}
unsafe impl Sync for GlobalFrameAllocator {}

unsafe impl FrameAllocator<Size4KiB> for GlobalFrameAllocator {
  fn allocate_frame(&mut self) -> Option<PhysFrame> {
    self.next += 1;
    self.usable_frames().nth(self.next - 1)
  }
}
