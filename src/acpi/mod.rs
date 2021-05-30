mod acpi;
mod rsdp;
mod sdt;

use acpi::AcpiHeader;

pub fn init(rsdp: u64) {
  let header = AcpiHeader::from_rsdp(rsdp);
  let (sdt, entries) = match header {
    AcpiHeader::Rsdt(sdt, entries) => (sdt, entries),
    AcpiHeader::Xsdt(sdt, entries) => (sdt, entries),
  };

  log::info!(
    "main descriptor located at {:#x} with signature '{}'",
    sdt.address(),
    sdt.signature(),
  );

  for entry in entries {
    log::info!("entry {:#x} with signature '{}'", entry.address(), entry.signature());
  }
}
