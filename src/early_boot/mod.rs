pub mod logger;

use crate::utils::locked::Locked;

use bootloader::boot_info::FrameBuffer;
use log::LevelFilter;
use logger::Logger;
use spin::Once;

static LOGGER: Once<Locked<Logger>> = Once::new();

// TODO: Make this a buffered logger + implement line scrolling
pub fn init_logger(fb: &'static mut FrameBuffer) {
  let logger = LOGGER.call_once(move || Locked::new(Logger::new(fb)));

  log::set_logger(logger).expect("logger has already been initialized");
  log::set_max_level(LevelFilter::Debug);
}
