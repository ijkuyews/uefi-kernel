#![no_std]
#![no_main]
#![feature(abi_x86_interrupt, alloc_error_handler, asm, lang_items, panic_info_message)]

mod acpi;
mod early_boot;
mod interrupts;
mod memory;
mod panic_handler;
mod utils;

use bootloader::{boot_info::KernelInfo, entry_point, BootInfo};
use spin::Once;
use x86_64::VirtAddr;

pub static KERNEL_INFO: Once<KernelInfo> = Once::new();
pub static PHYS_MEM_OFFSET: Once<VirtAddr> = Once::new();

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
  let mem_regions = &boot_info.memory_regions;
  let phys_mem_offset = boot_info
    .physical_memory_offset
    .into_option()
    .expect("cannot proceed without the physical memory offset");

  KERNEL_INFO.call_once(|| boot_info.kernel_info.clone());
  PHYS_MEM_OFFSET.call_once(|| VirtAddr::new(phys_mem_offset));

  if let Some(fb) = boot_info.framebuffer.as_mut() {
    let fb_abbr = fb.buffer().as_ptr() as u64;

    early_boot::init_logger(fb);

    log::info!("using framebuffer at {:#x}", fb_abbr);
  } else {
    // TODO: Use serial output for logging instead?
    panic!("cannot proceed without a framebuffer");
  }

  memory::init(phys_mem_offset, mem_regions);

  interrupts::init();

  log::info!("loaded the interrupt descriptor table");

  let rsdp_addr = boot_info
    .rsdp_addr
    .into_option()
    .expect("cannot proceed without the rsdp structure");

  log::info!("found rsdp structure at {:#x}", rsdp_addr);

  acpi::init(rsdp_addr);

  loop {
    unsafe { asm!("hlt") }
  }
}
