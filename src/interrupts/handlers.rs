use x86_64::{
  registers::control::Cr2,
  structures::idt::{InterruptStackFrame, PageFaultErrorCode},
};

pub extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _: u64) -> ! {
  panic!("double fault exception, stack frame: {:?}", stack_frame);
}

pub extern "x86-interrupt" fn general_protection_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) {
  panic!(
    "general protection fault exception, error code: {:#x}, stack frame: {:?}",
    error_code, stack_frame
  );
}

pub extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
  panic!(
    "page fault exception, accessed address: {:#x}, error code: {:?}, stack frame: {:?}",
    Cr2::read(),
    error_code,
    stack_frame
  );
}
