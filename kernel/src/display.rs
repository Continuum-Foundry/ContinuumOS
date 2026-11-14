//! Checked single-output boot scanout. Internal API; no frozen driver ABI.
use crate::{BootInfoV1, Framebuffer, Range, Result, device, error};

pub const MAX_WIDTH: u64 = 1920;
pub const MAX_HEIGHT: u64 = 1080;
pub const MAX_PITCH: u64 = 8192;
#[derive(Clone, Copy)]
pub struct Output {
    pub fb: Framebuffer,
    pub pages: Range,
}
impl Output {
    pub fn checked(info: &BootInfoV1) -> Result<Self> {
        info.validate()?;
        if info.framebuffer_count != 1 {
            return Err(error("display", "missing_or_multiple"));
        }
        let fb = info.framebuffers[0];
        if fb.width > MAX_WIDTH || fb.height > MAX_HEIGHT || fb.pitch > MAX_PITCH {
            return Err(error("display", "capacity"));
        }
        if fb.sizes != [8; 3] || !fb.shifts.iter().all(|s| s % 8 == 0) {
            return Err(error("display", "format"));
        }
        let base = fb.range.base & !4095;
        let end = fb
            .range
            .end()
            .checked_add(4095)
            .ok_or(error("display", "overflow"))?
            & !4095;
        let pages = Range::new(base, end - base, "display")?.physical("display")?;
        // Whole mapped pages, including edge bytes, must be framebuffer-owned.
        if !info.covered(pages, |k| k == 7)
            || info
                .regions()
                .iter()
                .any(|r| r.kind != 7 && r.range.overlaps(pages))
        {
            return Err(error("display", "reservation"));
        }
        if info.kernel.overlaps(pages)
            || info.executable_file.overlaps(pages)
            || info.modules[..info.module_count]
                .iter()
                .any(|m| m.range.overlaps(pages))
            || info
                .acpi_root
                .is_some_and(|a| pages.overlaps(Range { base: a.0, len: 36 }))
        {
            return Err(error("display", "overlap"));
        }
        Ok(Self { fb, pages })
    }
    pub fn pixel(self, rgb: [u8; 3]) -> u32 {
        (u32::from(rgb[0]) << self.fb.shifts[0])
            | (u32::from(rgb[1]) << self.fb.shifts[1])
            | (u32::from(rgb[2]) << self.fb.shifts[2])
    }
    /// One immutable copied RGBX row. Signed placement clips before pointer arithmetic.
    pub fn row(self, x: i64, y: i64, pixels: usize) -> Result<Option<Row>> {
        if pixels == 0 || pixels > MAX_WIDTH as usize {
            return Err(error("display", "row_capacity"));
        }
        if y < 0 || y as u64 >= self.fb.height {
            return Ok(None);
        }
        let end = x
            .checked_add(pixels as i64)
            .ok_or(error("display", "overflow"))?;
        let left = x.max(0);
        let right = end.min(self.fb.width as i64);
        if right <= left {
            return Ok(None);
        }
        Ok(Some(Row {
            skip: (left - x) as usize,
            count: (right - left) as usize,
            offset: y as u64 * self.fb.pitch + left as u64 * u64::from(self.fb.bpp / 8),
        }))
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct Row {
    pub skip: usize,
    pub count: usize,
    pub offset: u64,
}

/// P4-03 broker grants bind a trusted role to an exact process incarnation.
pub struct Lease {
    broker: device::Broker,
    active: Option<(device::Owner, device::Grant)>,
    generation: u64,
}
impl Lease {
    pub const EMPTY: Self = Self {
        broker: device::Broker::EMPTY,
        active: None,
        generation: 0,
    };
    pub fn bind(&mut self, owner: device::Owner, resource: Range) -> Result<u64> {
        if self.active.is_some() {
            return Err(error("display", "busy"));
        }
        let next = self
            .generation
            .checked_add(1)
            .ok_or(error("display", "generation"))?;
        let grant = self.broker.grant(
            owner,
            device::Resource::new(resource.base, resource.len, device::Kind::Mmio)?,
            device::MAP,
        )?;
        self.active = Some((owner, grant));
        self.generation = next;
        Ok(next)
    }
    pub fn check(&mut self, owner: device::Owner, generation: u64) -> Result<()> {
        let (_, grant) = self.active.ok_or(error("display", "unbound"))?;
        if generation != self.generation {
            return Err(error("display", "stale"));
        }
        self.broker.map(owner, grant, 0, 1)?;
        Ok(())
    }
    pub fn release(&mut self, owner: device::Owner) -> Result<()> {
        let (_, grant) = self.active.ok_or(error("display", "unbound"))?;
        self.broker.revoke(owner, grant)?;
        self.active = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn output(bpp: u16, shifts: [u8; 3]) -> Output {
        Output {
            fb: Framebuffer {
                width: 5,
                height: 3,
                pitch: 24,
                bpp,
                sizes: [8; 3],
                shifts,
                ..Framebuffer::EMPTY
            },
            pages: Range {
                base: 0xe0000000,
                len: 4096,
            },
        }
    }
    #[test]
    fn channel_conversion_clipping_padding_and_guards() {
        for (bpp, shifts) in [(24, [0, 8, 16]), (32, [16, 8, 0]), (32, [8, 16, 24])] {
            let o = output(bpp, shifts);
            let mut guarded = [0xadu8; 74];
            guarded[1..73].fill(0); // full initial clear includes padded rows
            for y in -1..4 {
                for x in [-9, -2, 0, 4, 5, i64::MAX] {
                    let before = guarded;
                    let row = o.row(x, y, 4);
                    if let Ok(Some(r)) = row {
                        for i in 0..r.count {
                            let at = 1 + r.offset as usize + i * (bpp / 8) as usize;
                            let p = o.pixel([255, 17, 64]).to_le_bytes();
                            guarded[at..at + (bpp / 8) as usize]
                                .copy_from_slice(&p[..(bpp / 8) as usize]);
                        }
                    }
                    assert_eq!(guarded[0], 0xad);
                    assert_eq!(guarded[73], 0xad);
                    if y < 0 || y >= 3 || x == 5 || x == -9 || x == i64::MAX {
                        assert_eq!(before, guarded);
                    }
                }
            }
            for y in 0..3 {
                assert!(
                    guarded[1 + y * 24 + 5 * (bpp / 8) as usize..1 + (y + 1) * 24]
                        .iter()
                        .all(|b| *b == 0)
                );
            }
        }
        assert!(output(32, [16, 8, 0]).row(0, 0, 1921).is_err());
    }
    #[test]
    fn incarnation_epoch_and_app_authority() {
        let mut l = Lease::EMPTY;
        let a = device::Owner {
            process: 1,
            generation: 1,
        };
        let b = device::Owner {
            process: 2,
            generation: 1,
        };
        let r = Range {
            base: 0xe0000000,
            len: 4096,
        };
        let g = l.bind(a, r).unwrap();
        assert!(l.bind(b, r).is_err());
        assert!(l.check(b, g).is_err());
        assert!(l.release(b).is_err());
        l.check(a, g).unwrap();
        l.release(a).unwrap();
        assert!(l.check(a, g).is_err());
        let a2 = device::Owner { generation: 2, ..a };
        let g2 = l.bind(a2, r).unwrap();
        assert_ne!(g, g2);
        assert!(l.check(a2, g).is_err());
        assert!(l.check(a, g2).is_err());
        l.check(a2, g2).unwrap();
    }
}
