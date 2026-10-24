//! Bounded selected hosted UI and retained comparison mode. Internal types are not a wire ABI.
#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::{string::String, vec::Vec};

pub mod input;
pub mod layout;
pub mod motion;
pub mod paint;
pub mod retained;
pub mod scene;
pub mod semantics;
pub mod surface;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Bounds,
    Invalid,
    Duplicate,
    Resource,
    Revision,
    Lease,
    Exhausted,
    Input,
    Model,
    Text,
}
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect(pub [f32; 4]);
impl Rect {
    pub fn validate(self) -> Result<()> {
        if self.0.iter().any(|v| !v.is_finite() || v.abs() > 16384.0)
            || self.0[2] < 0.0
            || self.0[3] < 0.0
        {
            return Err(Error::Bounds);
        }
        Ok(())
    }
    pub fn contains(self, x: f32, y: f32) -> bool {
        let [a, b, w, h] = self.0;
        x >= a && y >= b && x < a + w && y < b + h
    }
    pub fn intersect(self, other: Self) -> Self {
        let [a, b, w, h] = self.0;
        let [x, y, u, v] = other.0;
        let left = a.max(x);
        let top = b.max(y);
        Self([
            left,
            top,
            (a + w).min(x + u).max(left) - left,
            (b + h).min(y + v).max(top) - top,
        ])
    }
    pub fn union(self, other: Self) -> Self {
        let [a, b, w, h] = self.0;
        let [x, y, u, v] = other.0;
        Self([
            a.min(x),
            b.min(y),
            (a + w).max(x + u) - a.min(x),
            (b + h).max(y + v) - b.min(y),
        ])
    }
    pub fn expand(self, radius: f32) -> Self {
        let [x, y, w, h] = self.0;
        Self([x - radius, y - radius, w + 2.0 * radius, h + 2.0 * radius])
    }
    pub fn pixels(self, scale: u16, width: u32, height: u32) -> [u32; 4] {
        let s = scale as f32 / 100.0;
        let [x, y, w, h] = self.0;
        let floor = |v: f32| {
            let n = v as i32;
            if (n as f32) > v { n - 1 } else { n }
        };
        let ceil = |v: f32| {
            let n = v as i32;
            if (n as f32) < v { n + 1 } else { n }
        };
        [
            floor(x * s).clamp(0, width as i32) as u32,
            floor(y * s).clamp(0, height as i32) as u32,
            ceil((x + w) * s).clamp(0, width as i32) as u32,
            ceil((y + h) * s).clamp(0, height as i32) as u32,
        ]
    }
}
pub fn valid_id(s: &str) -> bool {
    !s.is_empty() && s.len() <= 256 && s.chars().count() <= 64
}

#[derive(Clone, Debug, PartialEq)]
pub struct Semantic {
    pub id: String,
    pub widget: String,
    pub role: String,
    pub name: String,
    pub value: String,
    pub action: String,
    pub bounds: Rect,
    pub focused: bool,
    pub checked: bool,
    pub logical_count: u32,
    pub realized: Vec<u32>,
}

#[cfg(test)]
mod selected_tests;
#[cfg(test)]
mod tests;
