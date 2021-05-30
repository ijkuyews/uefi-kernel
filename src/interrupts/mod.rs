mod handlers;

use spin::Once;
use x86_64::structures::idt::InterruptDescriptorTable;

static IDT: Once<InterruptDescriptorTable> = Once::new();

pub fn init() {
  let mut idt = InterruptDescriptorTable::new();

  idt.double_fault.set_handler_fn(handlers::double_fault_handler);
  idt
    .general_protection_fault
    .set_handler_fn(handlers::general_protection_fault_handler);
  idt.page_fault.set_handler_fn(handlers::page_fault_handler);

  IDT.call_once(|| idt).load();
}
