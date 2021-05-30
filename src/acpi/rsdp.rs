#[repr(C, packed)]
pub struct RsdpHeader {
  signature: [u8; 8],
  checksum: u8,
  oemid: [u8; 6],
  revision: u8,
  rsdt_address: u32,
  length: u32,
  xsdt_address: u64,
  extended_checksum: u8,
  reserved: [u8; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RsdtType {
  Rsdt,
  Xsdt,
}

impl RsdpHeader {
  pub fn sdt_type(&self) -> RsdtType {
    match self.revision {
      0 => RsdtType::Rsdt,
      2 => RsdtType::Xsdt,
      _ => panic!("invalid rdsp revision number: {}", self.revision),
    }
  }

  pub fn sdt_address(&self) -> u64 {
    match self.sdt_type() {
      RsdtType::Rsdt => self.rsdt_address as u64,
      RsdtType::Xsdt => self.xsdt_address,
    }
  }
}
