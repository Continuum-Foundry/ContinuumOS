#![no_std]
//! Internal, owned boot handoff. This is neither a wire format nor an allocator.
pub mod memory;
pub mod owned_memory;
pub mod platform;
pub mod scheduler;
pub mod user_boundary;
pub mod user_image;

pub const MAX_REGIONS: usize = 512;
pub const MAX_MODULES: usize = 16;
pub const MAX_FRAMEBUFFERS: usize = 4;
pub const REQUIRED: u8 = 0x1f; // base revision, memory map, HHDM, executable address/file

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Error {
    pub field: &'static str,
    pub reason: &'static str,
}
pub type Result<T> = core::result::Result<T, Error>;
pub const fn error(field: &'static str, reason: &'static str) -> Error {
    Error { field, reason }
}
pub fn count(value: u64, capacity: usize, field: &'static str) -> Result<usize> {
    if value > capacity as u64 {
        return Err(error(field, "capacity"));
    }
    Ok(value as usize)
}
pub fn revision(value: u64, maximum: u64, field: &'static str) -> Result<()> {
    if value > maximum {
        return Err(error(field, "revision"));
    }
    Ok(())
}
pub fn canonical(value: u64) -> bool {
    // The bootstrap adapter requests four-level paging explicitly.
    value <= 0x0000_7fff_ffff_ffff || value >= 0xffff_8000_0000_0000
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysAddr(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VirtAddr(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range {
    pub base: u64,
    pub len: u64,
}
impl Range {
    pub const EMPTY: Self = Self { base: 0, len: 0 };
    pub fn new(base: u64, len: u64, field: &'static str) -> Result<Self> {
        if len == 0 {
            return Err(error(field, "zero_length"));
        }
        base.checked_add(len).ok_or(error(field, "overflow"))?;
        Ok(Self { base, len })
    }
    pub fn physical(self, field: &'static str) -> Result<Self> {
        let r = Self::new(self.base, self.len, field)?;
        if r.end() > (1u64 << 52) {
            return Err(error(field, "physical_address"));
        }
        Ok(r)
    }
    pub fn end(self) -> u64 {
        self.base + self.len
    }
    pub fn contains(self, other: Self) -> bool {
        self.base <= other.base && other.end() <= self.end()
    }
    pub fn overlaps(self, other: Self) -> bool {
        self.base < other.end() && other.base < self.end()
    }
}
pub fn virtual_range(base: u64, len: u64, field: &'static str) -> Result<Range> {
    let r = Range::new(base, len, field)?;
    if !canonical(base) || !canonical(r.end() - 1) || (base >> 47 != (r.end() - 1) >> 47) {
        return Err(error(field, "virtual_address"));
    }
    Ok(r)
}
pub fn from_hhdm(address: u64, len: u64, offset: u64, field: &'static str) -> Result<Range> {
    virtual_range(address, len, field)?;
    Range::new(
        address.checked_sub(offset).ok_or(error(field, "hhdm"))?,
        len,
        field,
    )?
    .physical(field)
}

#[derive(Clone, Copy)]
pub struct Text<const N: usize> {
    bytes: [u8; N],
    len: usize,
}
impl<const N: usize> Default for Text<N> {
    fn default() -> Self {
        Self::EMPTY
    }
}
impl<const N: usize> Text<N> {
    pub const EMPTY: Self = Self {
        bytes: [0; N],
        len: 0,
    };
    pub fn copy_cstr(&mut self, source: &[u8], field: &'static str) -> Result<()> {
        let end = source
            .iter()
            .take(N)
            .position(|&b| b == 0)
            .ok_or(error(field, "unterminated"))?;
        if source[..end]
            .iter()
            .any(|b| !b.is_ascii() || b.is_ascii_control())
        {
            return Err(error(field, "encoding"));
        }
        self.bytes[..end].copy_from_slice(&source[..end]);
        self.len = end;
        Ok(())
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(self.bytes()).unwrap_or("")
    }
}
#[derive(Clone, Copy)]
pub struct Region {
    pub range: Range,
    pub kind: u64,
}
impl Region {
    pub const EMPTY: Self = Self {
        range: Range::EMPTY,
        kind: 1,
    };
}
#[derive(Clone, Copy)]
pub struct Module {
    pub range: Range,
    pub path: Text<256>,
    pub arguments: Text<256>,
}
impl Module {
    pub const EMPTY: Self = Self {
        range: Range::EMPTY,
        path: Text::EMPTY,
        arguments: Text::EMPTY,
    };
}
#[derive(Clone, Copy)]
pub struct Framebuffer {
    pub address: VirtAddr,
    pub range: Range,
    pub width: u64,
    pub height: u64,
    pub pitch: u64,
    pub bpp: u16,
    pub model: u8,
    pub sizes: [u8; 3],
    pub shifts: [u8; 3],
}
impl Framebuffer {
    pub const EMPTY: Self = Self {
        address: VirtAddr(0),
        range: Range::EMPTY,
        width: 0,
        height: 0,
        pitch: 0,
        bpp: 0,
        model: 0,
        sizes: [0; 3],
        shifts: [0; 3],
    };
    pub fn validate(&self, hhdm: u64) -> Result<Range> {
        if self.width == 0 || self.height == 0 {
            return Err(error("framebuffer", "dimensions"));
        }
        if self.model != 1 || ![24, 32].contains(&self.bpp) {
            return Err(error("framebuffer", "format"));
        }
        let row = self
            .width
            .checked_mul(u64::from(self.bpp / 8))
            .ok_or(error("framebuffer", "overflow"))?;
        if self.pitch < row {
            return Err(error("framebuffer", "pitch"));
        }
        let len = self
            .pitch
            .checked_mul(self.height)
            .ok_or(error("framebuffer", "overflow"))?;
        let mut used = 0u64;
        for i in 0..3 {
            let size = u32::from(self.sizes[i]);
            let shift = u32::from(self.shifts[i]);
            if size == 0 || size + shift > u32::from(self.bpp) {
                return Err(error("framebuffer", "format"));
            }
            let bits = ((1u64 << size) - 1) << shift;
            if used & bits != 0 {
                return Err(error("framebuffer", "format"));
            }
            used |= bits;
        }
        let r = from_hhdm(self.address.0, len, hhdm, "framebuffer")?;
        if r != self.range {
            return Err(error("framebuffer", "range"));
        }
        Ok(r)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReservationReason {
    Kernel,
    ExecutableFile,
    Module,
    Framebuffer,
    LoaderLive,
    Acpi,
    Firmware,
}

pub struct BootInfoV1 {
    pub version: u32,
    pub prerequisites: u8,
    pub hhdm: VirtAddr,
    pub regions: [Region; MAX_REGIONS],
    pub region_count: usize,
    pub kernel: Range,
    pub executable_file: Range,
    pub executable_path: Text<256>,
    pub arguments: Text<1024>,
    pub loader_name: Text<256>,
    pub loader_version: Text<256>,
    pub modules: [Module; MAX_MODULES],
    pub module_count: usize,
    pub framebuffers: [Framebuffer; MAX_FRAMEBUFFERS],
    pub framebuffer_count: usize,
    pub acpi_root: Option<PhysAddr>,
}
impl BootInfoV1 {
    pub const EMPTY: Self = Self {
        version: 1,
        prerequisites: 0,
        hhdm: VirtAddr(0),
        regions: [Region::EMPTY; MAX_REGIONS],
        region_count: 0,
        kernel: Range::EMPTY,
        executable_file: Range::EMPTY,
        executable_path: Text::EMPTY,
        arguments: Text::EMPTY,
        loader_name: Text::EMPTY,
        loader_version: Text::EMPTY,
        modules: [Module::EMPTY; MAX_MODULES],
        module_count: 0,
        framebuffers: [Framebuffer::EMPTY; MAX_FRAMEBUFFERS],
        framebuffer_count: 0,
        acpi_root: None,
    };
    pub fn regions(&self) -> &[Region] {
        &self.regions[..self.region_count]
    }
    /// Cover a physical interval by a union of permitted descriptors. Never infer
    /// that reserved/MMIO memory is accessible merely from its presence here.
    pub fn covered(&self, range: Range, permitted: impl Fn(u64) -> bool) -> bool {
        let mut cursor = range.base;
        while cursor < range.end() {
            let next = self
                .regions()
                .iter()
                .filter(|r| permitted(r.kind) && r.range.base <= cursor && cursor < r.range.end())
                .map(|r| r.range.end())
                .max()
                .unwrap_or(cursor);
            if next == cursor {
                return false;
            }
            cursor = next;
        }
        true
    }
    pub fn validate_memory(&self) -> Result<()> {
        count(self.region_count as u64, MAX_REGIONS, "memory")?;
        if self.region_count == 0 {
            return Err(error("memory", "missing"));
        }
        for (i, a) in self.regions().iter().enumerate() {
            a.range.physical("memory")?;
            if [0, 5].contains(&a.kind) && (a.range.base % 4096 != 0 || a.range.len % 4096 != 0) {
                return Err(error("memory", "alignment"));
            }
            if i > 0 && self.regions[i - 1].range.base > a.range.base {
                return Err(error("memory", "order"));
            }
            for b in &self.regions[..i] {
                if a.range.overlaps(b.range)
                    && ([0, 5].contains(&a.kind) || [0, 5].contains(&b.kind))
                {
                    return Err(error("memory", "overlap"));
                }
            }
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<()> {
        revision(self.version as u64, 1, "boot_info")?;
        if self.version != 1 {
            return Err(error("boot_info", "revision"));
        }
        for (bit, field) in [
            (1, "base"),
            (2, "memory"),
            (4, "hhdm"),
            (8, "executable_address"),
            (16, "executable_file"),
        ] {
            if self.prerequisites & bit == 0 {
                return Err(error(field, "missing"));
            }
        }
        if self.prerequisites != REQUIRED {
            return Err(error("required", "flags"));
        }
        if self.hhdm.0 < 0xffff_8000_0000_0000 || self.hhdm.0 % 4096 != 0 {
            return Err(error("hhdm", "virtual_address"));
        }
        self.validate_memory()?;
        for (r, field) in [
            (self.kernel, "kernel"),
            (self.executable_file, "executable_file"),
        ] {
            r.physical(field)?;
            // The raw ELF file is loader-associated data, distinct from loaded
            // PT_LOAD segments. Both classes stay reserved throughout this story.
            if !self.covered(r, |k| k == 6 || (field == "executable_file" && k == 5)) {
                return Err(error(field, "backing"));
            }
        }
        count(self.module_count as u64, MAX_MODULES, "modules")?;
        for m in &self.modules[..self.module_count] {
            m.range.physical("module")?;
            if !self.covered(m.range, |k| k == 6) {
                return Err(error("module", "backing"));
            }
            if m.range.overlaps(self.kernel) || m.range.overlaps(self.executable_file) {
                return Err(error("module", "overlap"));
            }
        }
        for i in 0..self.module_count {
            for b in &self.modules[..i] {
                if self.modules[i].range.overlaps(b.range) {
                    return Err(error("module", "overlap"));
                }
            }
        }
        count(
            self.framebuffer_count as u64,
            MAX_FRAMEBUFFERS,
            "framebuffers",
        )?;
        for fb in &self.framebuffers[..self.framebuffer_count] {
            let r = fb.validate(self.hhdm.0)?;
            if !self.covered(r, |k| k == 7) {
                return Err(error("framebuffer", "backing"));
            }
            if self
                .regions()
                .iter()
                .any(|m| [0, 5, 6].contains(&m.kind) && m.range.overlaps(r))
            {
                return Err(error("framebuffer", "overlap"));
            }
        }
        if let Some(PhysAddr(address)) = self.acpi_root {
            Range::new(address, 36, "acpi")?.physical("acpi")?;
            // The Limine locator has no alignment guarantee. OVMF's extended
            // RSDP can be at legacy-RSDP + 20; keep a byte address, never a &T.
            if address == 0 {
                return Err(error("acpi", "physical_address"));
            }
        }
        Ok(())
    }
    /// Enumerate reservations without constructing another large fixed array.
    /// Overlapping reservations are deliberate; no ranges are made allocatable.
    pub fn reservations(&self, mut visit: impl FnMut(Range, ReservationReason)) {
        visit(self.kernel, ReservationReason::Kernel);
        visit(self.executable_file, ReservationReason::ExecutableFile);
        for m in &self.modules[..self.module_count] {
            visit(m.range, ReservationReason::Module);
        }
        for fb in &self.framebuffers[..self.framebuffer_count] {
            visit(fb.range, ReservationReason::Framebuffer);
        }
        for r in self.regions() {
            if r.kind != 0 {
                visit(
                    r.range,
                    match r.kind {
                        5 => ReservationReason::LoaderLive,
                        2 | 3 => ReservationReason::Acpi,
                        _ => ReservationReason::Firmware,
                    },
                );
            }
        }
        if let Some(PhysAddr(p)) = self.acpi_root {
            visit(Range { base: p, len: 36 }, ReservationReason::Acpi);
        }
    }
}

/// Exact host-owned configuration, not a security credential. IDs are immutable
/// once copied; fault names are interpreted only by an explicitly gated binary.
pub fn identity(args: &Text<1024>) -> Result<(&str, &str)> {
    let mut words = args.as_str().split(' ');
    let session = words
        .next()
        .and_then(|s| s.strip_prefix("session="))
        .ok_or(error("session", "syntax"))?;
    if session.len() != 64
        || !session
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(error("session", "syntax"));
    }
    let fixture = words
        .next()
        .and_then(|s| s.strip_prefix("fixture="))
        .ok_or(error("fixture", "syntax"))?;
    if fixture.is_empty()
        || fixture.len() > 64
        || !fixture.bytes().all(|b| b.is_ascii_lowercase() || b == b'-')
        || words.next().is_some()
    {
        return Err(error("fixture", "syntax"));
    }
    Ok((session, fixture))
}

#[cfg(test)]
mod tests;
