// SPDX-License-Identifier: LGPL-2.1
// Copyright 2021 Daniel Vogelbacher <daniel@chaospixel.com>

pub mod gamma;
pub mod matrix;
pub mod raw;
pub mod sensor;
pub mod spline;
pub mod srgb;
pub mod xyz;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, String>;

/*
macro_rules! max {
  ($x: expr) => ($x);
  ($x: expr, $($z: expr),+) => {{
      let y = max!($($z),*);
      if $x > y {
          $x
      } else {
          y
      }
  }}
}

macro_rules! min {
  ($x: expr) => ($x);
  ($x: expr, $($z: expr),+) => {{
      let y = min!($($z),*);
      if $x < y {
          $x
      } else {
          y
      }
  }}
}
 */

/// Descriptor of a two-dimensional area
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Dim2 {
  pub w: usize,
  pub h: usize,
}

impl Dim2 {
  pub fn new(w: usize, h: usize) -> Self {
    Self { w, h }
  }

  pub fn is_empty(&self) -> bool {
    self.w == 0 && self.h == 0
  }
}

/// Rescale to u16 value
pub fn rescale_f32_to_u16(input: &[f32], black: u16, white: u16) -> Vec<u16> {
  if black == 0 {
    input.par_iter().map(|p| (p * white as f32) as u16).collect()
  } else {
    input.par_iter().map(|p| (p * (white - black) as f32) as u16 + black).collect()
  }
}

/// Rescale to u8 value
pub fn rescale_f32_to_u8(input: &[f32], black: u8, white: u8) -> Vec<u8> {
  if black == 0 {
    input.par_iter().map(|p| (p * white as f32) as u8).collect()
  } else {
    input.par_iter().map(|p| (p * (white - black) as f32) as u8 + black).collect()
  }
}

/// Clip a value with min/max value
pub fn clip(p: f32, min: f32, max: f32) -> f32 {
  if p > max {
    max
  } else if p < min {
    min
  } else if p.is_nan() {
    min
  } else {
    p
  }
}

/// A simple x/y point
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Point {
  pub x: usize,
  pub y: usize,
}

impl Point {
  pub fn new(x: usize, y: usize) -> Self {
    Self { x, y }
  }

  pub fn zero() -> Self {
    Self { x: 0, y: 0 }
  }
}

/// Rectangle by a point and dimension
#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Rect {
  pub p: Point,
  pub d: Dim2,
}

impl std::fmt::Debug for Rect {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    //f.debug_struct("Rect").field("p", &self.p).field("d", &self.d).finish()?;
    f.write_fmt(format_args!(
      "Rect{{{}:{}, {}x{}, LTRB=[{}, {}, {}, {}]}}",
      self.p.x,
      self.p.y,
      self.d.w,
      self.d.h,
      self.p.x,
      self.p.y,
      self.p.x + self.d.w,
      self.p.y + self.d.h
    ))
  }
}

impl Rect {
  pub fn new(p: Point, d: Dim2) -> Self {
    Self { p, d }
  }

  // left, top, right, bottom
  pub fn new_with_points(p1: Point, p2: Point) -> Self {
    Self {
      p: p1,
      d: Dim2 {
        w: p2.x - p1.x,
        h: p2.y - p1.y,
      },
    }
  }

  pub fn new_with_borders(dim: Dim2, borders: &[usize; 4]) -> Self {
    Self::new_with_points(Point::new(borders[0], borders[1]), Point::new(dim.w - borders[2], dim.h - borders[3]))
  }

  pub fn is_empty(&self) -> bool {
    self.d.is_empty()
  }

  /// Return in LTRB coordinates
  pub fn as_ltrb(&self) -> [usize; 4] {
    [self.p.x, self.p.y, self.p.x + self.d.w, self.p.y + self.d.h]
  }

  /// Return in TLBR
  pub fn as_tlbr(&self) -> [usize; 4] {
    [self.p.y, self.p.x, self.p.y + self.d.h, self.p.x + self.d.w]
  }
}

/// Crop image to specific area
pub fn crop<T: Clone>(input: &[T], dim: Dim2, area: Rect) -> Vec<T> {
  let mut output = Vec::with_capacity(area.d.h * area.d.w);
  output.extend(
    input
      .chunks_exact(dim.w)
      .skip(area.p.y)
      .take(area.d.h)
      .flat_map(|row| row[area.p.x..area.p.x + area.d.w].iter())
      .cloned(),
  );
  output
}
