use crate::{KERNEL_INFO, PHYS_MEM_OFFSET};

use core::{
  panic::PanicInfo,
  sync::atomic::{AtomicBool, Ordering},
};
use xmas_elf::{
  sections::{SectionData, ShType},
  symbol_table::Entry,
  ElfFile,
};

static BACKTRACE: AtomicBool = AtomicBool::new(true);

#[repr(C)]
struct StackFrame {
  previous: *const StackFrame,
  return_addr: usize,
}

struct BacktraceGuard {
  previous: bool,
}

impl BacktraceGuard {
  pub fn new() -> Self {
    Self {
      previous: BACKTRACE.swap(false, Ordering::Relaxed),
    }
  }

  pub fn enabled(self) -> bool {
    self.previous
  }
}

impl Drop for BacktraceGuard {
  fn drop(&mut self) {
    if self.previous {
      BACKTRACE.store(true, Ordering::Relaxed);
    }
  }
}

#[panic_handler]
extern "C" fn rust_begin_unwind(info: &PanicInfo) -> ! {
  let backtrace = BacktraceGuard::new();

  match (info.location(), info.message()) {
    (Some(loc), Some(message)) => log::error!("kernel panicked at {}:{}: {}", loc.file(), loc.line(), message),
    (Some(loc), None) => log::error!("kernel panicked at {}:{}", loc.file(), loc.line()),
    (None, Some(message)) => log::error!("kernel panicked: {}", message),
    (None, None) => log::error!("kernel panicked, no idea where, neither why, you're on your own!"),
  }

  if backtrace.enabled() {
    let kernel_info = KERNEL_INFO.get().expect("no kernel info was located");
    let phys_memory_offset = PHYS_MEM_OFFSET.get().expect("how did we get here?").as_u64();

    let kernel_data = unsafe {
      core::slice::from_raw_parts(
        (kernel_info.kernel_base + phys_memory_offset) as *const u8,
        kernel_info.kernel_size as usize,
      )
    };

    let kernel_file = ElfFile::new(kernel_data).expect("could not read kernel binary");
    let symbols_data = kernel_file
      .section_iter()
      .find(|sect| sect.get_type() == Ok(ShType::SymTab))
      .map(|sect| sect.get_data(&kernel_file))
      .unwrap();

    if let SectionData::SymbolTable64(symbol_table) = symbols_data.unwrap() {
      let mut stack_frame: *const StackFrame;

      unsafe { asm!("mov {}, rbp", out(reg) stack_frame) }

      if stack_frame.is_null() {
        log::error!("frame pointers were not emitted for this build, cannot print backtrace");
      } else {
        for _ in 0..64 {
          let stack_frame_ref = unsafe { &*stack_frame };
          let return_addr = stack_frame_ref.return_addr as u64;

          if return_addr == 0 {
            break;
          }

          stack_frame = stack_frame_ref.previous;

          for entry in symbol_table {
            let start = entry.value();
            let end = start + entry.size();

            if (start..=end).contains(&return_addr) {
              let mangled_name = entry.get_name(&kernel_file).expect("could not get symbol name");
              let demangled_name = rustc_demangle::demangle(mangled_name);

              log::error!("{:#x}: {}", return_addr, demangled_name);
            }
          }
        }
      }
    } else {
      panic!("symbol section data does not contain the symbol table");
    }
  }

  loop {
    unsafe { asm!("hlt") }
  }
}

#[allow(non_snake_case)]
#[no_mangle]
extern "C" fn _Unwind_Resume(_: usize) -> ! {
  loop {
    unsafe { asm!("hlt") }
  }
}

#[lang = "eh_personality"]
#[no_mangle]
extern "C" fn rust_eh_personality() -> ! {
  loop {
    unsafe { asm!("hlt") }
  }
}
