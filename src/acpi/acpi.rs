use crate::memory::identity_map_pages;

use super::{
  rsdp::{RsdpHeader, RsdtType},
  sdt::SdtHeader,
};

use x86_64::structures::paging::PageTableFlags;

pub enum AcpiHeader {
  Rsdt(&'static SdtHeader, AcpiTableIterator),
  Xsdt(&'static SdtHeader, AcpiTableIterator),
}

#[derive(Clone)]
pub struct AcpiTableIterator {
  kind: RsdtType,
  sdt: &'static SdtHeader,
  entries: usize,
  current: usize,
}

impl AcpiTableIterator {
  pub fn new(kind: RsdtType, sdt: &'static SdtHeader, entries: usize) -> Self {
    Self {
      kind,
      sdt,
      entries,
      current: 0,
    }
  }

  pub fn from_rsdt(sdt: &'static SdtHeader) -> Self {
    Self::new(RsdtType::Rsdt, sdt, sdt.data_length() / 4)
  }

  pub fn from_xsdt(sdt: &'static SdtHeader) -> Self {
    Self::new(RsdtType::Xsdt, sdt, sdt.data_length() / 8)
  }
}

impl Iterator for AcpiTableIterator {
  type Item = &'static SdtHeader;

  fn next(&mut self) -> Option<Self::Item> {
    if self.current >= self.entries {
      return None;
    }

    let entry = match self.kind {
      RsdtType::Rsdt => self.sdt.data_address() + self.current as u64 * 4,
      RsdtType::Xsdt => self.sdt.data_address() + self.current as u64 * 8,
    };

    self.current += 1;

    Some(SdtHeader::from_addr(entry))
  }
}

impl AcpiHeader {
  pub fn from_rsdp(rsdp: u64) -> Self {
    if let Err(_) = identity_map_pages(rsdp, rsdp + 1, PageTableFlags::PRESENT, true) {
      log::trace!("page with the rsdp was already identity mapped");
    }

    let rsdp = unsafe { &*(rsdp as *const RsdpHeader) };
    let sdt = SdtHeader::from_addr(rsdp.sdt_address());

    match rsdp.sdt_type() {
      RsdtType::Rsdt => AcpiHeader::Rsdt(sdt, AcpiTableIterator::from_rsdt(sdt)),
      RsdtType::Xsdt => AcpiHeader::Xsdt(sdt, AcpiTableIterator::from_xsdt(sdt)),
    }
  }
}
