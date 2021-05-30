use crate::utils::locked::Locked;

use bootloader::boot_info::{FrameBuffer, FrameBufferInfo, PixelFormat};
use core::fmt::{Result as FmtResult, Write};
use font8x8::{UnicodeFonts, BASIC_FONTS, BLOCK_UNICODE};
use log::Log;

#[derive(Clone, Copy, Debug)]
pub struct Color {
  pub r: u8,
  pub g: u8,
  pub b: u8,
}

impl Color {
  pub const fn new(r: u8, g: u8, b: u8) -> Self {
    Self { r, g, b }
  }
}

impl From<u32> for Color {
  fn from(color: u32) -> Self {
    Self {
      r: ((color >> 16) & 0xff) as u8,
      g: ((color >> 8) & 0xff) as u8,
      b: (color & 0xff) as u8,
    }
  }
}

pub struct Logger {
  buffer: &'static mut [u8],
  info: FrameBufferInfo,
  color: Color,
  col: usize,
  row: usize,
}

impl Logger {
  pub fn new(fb: &'static mut FrameBuffer) -> Self {
    let info = fb.info();
    let buffer = fb.buffer_mut();

    let mut res = Self {
      buffer,
      info,
      color: Color::new(255, 255, 255),
      col: 0,
      row: 0,
    };

    res.clear_screen();
    res
  }

  pub fn set_color(&mut self, color: Color) {
    self.color = color;
  }

  pub fn write_char(&mut self, ch: char) {
    if ch == '\n' {
      self.new_line();
    } else {
      if self.col >= self.info.horizontal_resolution {
        self.new_line();
      }

      let character = match BASIC_FONTS.get(ch) {
        Some(character) => character,
        None => BLOCK_UNICODE[8].byte_array(),
      };

      for (y, &byte) in character.iter().enumerate() {
        for x in 0..8 {
          if byte & (1 << x) != 0 {
            self.write_pixel(self.col + x, self.row + y);
          }
        }
      }

      self.col += 8;
    }
  }

  fn clear_screen(&mut self) {
    unsafe {
      self.buffer.as_mut_ptr().write_bytes(0, self.info.byte_len);
    }
  }

  fn new_line(&mut self) {
    self.col = 0;
    self.row += 12;

    if self.row >= (self.info.vertical_resolution - 8) {
      self.clear_screen();

      self.row = 0;
    }
  }

  fn write_pixel(&mut self, x: usize, y: usize) {
    let color = match self.info.pixel_format {
      PixelFormat::RGB => [self.color.r, self.color.g, self.color.b, 0_u8],
      PixelFormat::BGR => [self.color.b, self.color.g, self.color.r, 0_u8],
      _ => panic!("unsupported frame buffer format"),
    };

    let pixel_offset = y * self.info.stride + x;
    let byte_offset = pixel_offset * 4;

    self.buffer[byte_offset..byte_offset + 4].copy_from_slice(&color[..4]);
  }
}

impl Log for Locked<Logger> {
  fn enabled(&self, _: &log::Metadata) -> bool {
    true
  }

  fn log(&self, record: &log::Record) {
    let mut logger = self.lock();

    logger.set_color(match record.level() {
      log::Level::Trace => Color::from(0x76_26_71),
      log::Level::Debug => Color::from(0x39_b5_4a),
      log::Level::Info => Color::from(0xff_ff_ff),
      log::Level::Warn => Color::from(0xff_c7_06),
      log::Level::Error => Color::from(0xff_00_00),
    });

    write!(logger, "{:<5}", record.level()).unwrap();

    logger.set_color(Color::from(0xff_ff_ff));

    write!(logger, " - {}\n", record.args()).unwrap();
  }

  fn flush(&self) {}
}

impl Write for Logger {
  fn write_str(&mut self, string: &str) -> FmtResult {
    for ch in string.chars() {
      self.write_char(ch);
    }

    Ok(())
  }
}
