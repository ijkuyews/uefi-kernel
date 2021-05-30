use crate::memory::identity_map_pages;

use core::mem::size_of;
use x86_64::structures::paging::PageTableFlags;

#[repr(C, packed)]
pub struct SdtHeader {
  signature: [u8; 4],
  length: u32,
  revision: u8,
  checksum: u8,
  oem_id: [u8; 6],
  oem_table_id: [u8; 8],
  oem_revision: u32,
  creator_id: u32,
  creator_revision: u32,
}

impl SdtHeader {
  pub fn from_addr(addr: u64) -> &'static SdtHeader {
    if let Err(_) = identity_map_pages(addr, addr + 1, PageTableFlags::PRESENT, true) {
      log::trace!("page with the sdt at {:#x} was already identity mapped", addr);
    }

    unsafe { &*(addr as *const Self) }
  }

  pub fn address(&self) -> u64 {
    self as *const Self as u64
  }

  pub fn data_address(&self) -> u64 {
    self.address() + size_of::<Self>() as u64
  }

  pub fn data_length(&self) -> usize {
    self.length as usize - size_of::<Self>()
  }

  pub fn signature(&self) -> &str {
    // .expect("invalid utf-8 sequence inside sdt signature")
    unsafe { core::str::from_utf8_unchecked(&self.signature) }
  }
}
