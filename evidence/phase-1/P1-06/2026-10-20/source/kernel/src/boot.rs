//! Limine base-3 adapter, private to this executable.
//!
//! Trusted bootstrap: the pinned loader/firmware must actually supply readable
//! response headers and memory-map objects in HHDM-mapped loader memory. Arithmetic
//! and canonicality checks are NOT proof that arbitrary pointers are mapped.
//! After collecting/validating the map we check every subsequent metadata span
//! against BOOTLOADER_RECLAIMABLE, and retroactively check bootstrap spans. No
//! arbitrary-pointer fault recovery, page-table walk, or memory reclamation occurs.
use continuum_kernel::*;
use core::{
    cell::UnsafeCell,
    mem::{align_of, size_of},
    ptr,
};

#[repr(C)]
struct Request {
    id: [u64; 4],
    revision: u64,
    response: UnsafeCell<u64>,
}
unsafe impl Sync for Request {} // loader writes before entry; single CPU, IF clear
impl Request {
    const fn new(a: u64, b: u64) -> Self {
        Self {
            id: [0xc7b1dd30df4c8b88, 0x0a82e883a194f07b, a, b],
            revision: 0,
            response: UnsafeCell::new(0),
        }
    }
    fn address(&self) -> u64 {
        unsafe { ptr::read_volatile(self.response.get()) }
    }
}
macro_rules! request {
    ($name:ident, $a:expr, $b:expr) => {
        #[used]
        #[unsafe(link_section = ".limine_requests")]
        static $name: Request = Request::new($a, $b);
    };
}
request!(HHDM, 0x48dcf1cb8ad2b852, 0x63984e959a98244b);
request!(MEMORY, 0x67cf3d9d378a806f, 0xe304acdfc50c3c62);
request!(EXEC_ADDRESS, 0x71ba76863cc55f63, 0xb2644a48c516a487);
request!(EXEC_FILE, 0xad97e90e83f1ed67, 0x31eb5d1c5ff23b69);
request!(MODULES, 0x3e7e279702be32af, 0xca1c4f3bd1280cee);
request!(FRAMEBUFFERS, 0x9d5827dcd881dd75, 0xa3148604f6fab11b);
request!(ACPI, 0xc5e77b6b397e7b43, 0x27637845accdcf3c);
request!(LOADER, 0xf55038d8e2a1202f, 0x279426fcf5f59740);

// Explicit four-level mode; revision-1 min=max=0 prevents five-level addresses.
#[repr(C)]
struct PagingRequest {
    request: Request,
    mode: u64,
    max: u64,
    min: u64,
}
unsafe impl Sync for PagingRequest {}
#[used]
#[unsafe(link_section = ".limine_requests")]
static PAGING: PagingRequest = PagingRequest {
    request: Request {
        revision: 1,
        ..Request::new(0x95c1a0edab0944cb, 0xa4e5cb3842f7488a)
    },
    mode: 0,
    max: 0,
    min: 0,
};

// Only prefixes actually consumed by P1-03. Values/offsets are the pinned
// limine.h ABI, not Rust binding private layout. All pointer fields stay integers.
#[repr(C)]
#[derive(Clone, Copy)]
struct Pair {
    revision: u64,
    value: u64,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct Array {
    revision: u64,
    count: u64,
    pointer: u64,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct RawRegion {
    base: u64,
    len: u64,
    kind: u64,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct RawFile {
    revision: u64,
    address: u64,
    size: u64,
    path: u64,
    arguments: u64,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct RawFramebuffer {
    address: u64,
    width: u64,
    height: u64,
    pitch: u64,
    bpp: u16,
    model: u8,
    red_size: u8,
    red_shift: u8,
    green_size: u8,
    green_shift: u8,
    blue_size: u8,
    blue_shift: u8,
}
const _: () = {
    assert!(size_of::<Request>() == 48);
    assert!(core::mem::offset_of!(Request, response) == 40);
    assert!(size_of::<PagingRequest>() == 72);
    assert!(size_of::<Pair>() == 16);
    assert!(size_of::<Array>() == 24);
    assert!(size_of::<RawRegion>() == 24);
    assert!(size_of::<RawFile>() == 40);
    assert!(core::mem::offset_of!(RawFile, arguments) == 32);
    assert!(size_of::<RawFramebuffer>() == 48);
    assert!(core::mem::offset_of!(RawFramebuffer, bpp) == 32);
};

fn numeric<T>(address: u64, field: &'static str) -> Result<()> {
    if address == 0 {
        return Err(error(field, "missing"));
    }
    virtual_range(address, size_of::<T>() as u64, field)?;
    if address % align_of::<T>() as u64 != 0 {
        return Err(error(field, "alignment"));
    }
    Ok(())
}
// Safety: caller supplies the trusted bootstrap object or a map-checked span.
unsafe fn bootstrap<T: Copy>(address: u64, field: &'static str) -> Result<T> {
    numeric::<T>(address, field)?;
    Ok(unsafe { ptr::read(address as *const T) })
}
fn span(info: &BootInfoV1, address: u64, len: u64, field: &'static str) -> Result<()> {
    let r = from_hhdm(address, len, info.hhdm.0, field)?;
    if !info.covered(r, |k| k == 5) {
        return Err(error(field, "metadata_backing"));
    }
    Ok(())
}
fn load<T: Copy>(info: &BootInfoV1, address: u64, field: &'static str) -> Result<T> {
    numeric::<T>(address, field)?;
    span(info, address, size_of::<T>() as u64, field)?;
    // Safety: pinned loader object, size/alignment/arithmetic and backing checked.
    unsafe { bootstrap(address, field) }
}
fn required(info: &BootInfoV1, request: &Request, field: &'static str) -> Result<Pair> {
    let p: Pair = load(info, request.address(), field)?;
    revision(p.revision, 0, field)?;
    Ok(p)
}
fn array(
    info: &BootInfoV1,
    request: &Request,
    capacity: usize,
    max_revision: u64,
    field: &'static str,
) -> Result<Option<Array>> {
    if request.address() == 0 {
        return Ok(None);
    }
    let a: Array = load(info, request.address(), field)?;
    revision(a.revision, max_revision, field)?;
    let n = count(a.count, capacity, field)?;
    if n == 0 {
        return Err(error(field, "zero_length"));
    }
    numeric::<u64>(a.pointer, field)?;
    span(
        info,
        a.pointer,
        a.count.checked_mul(8).ok_or(error(field, "overflow"))?,
        field,
    )?;
    Ok(Some(a))
}
fn string<const N: usize>(info: &BootInfoV1, address: u64, field: &'static str) -> Result<Text<N>> {
    let mut bytes = [0u8; N];
    for (i, byte) in bytes.iter_mut().enumerate() {
        let p = address
            .checked_add(i as u64)
            .ok_or(error(field, "overflow"))?;
        *byte = load::<u8>(info, p, field)?;
        if *byte == 0 {
            let mut text = Text::EMPTY;
            text.copy_cstr(&bytes[..=i], field)?;
            return Ok(text);
        }
    }
    Err(error(field, "unterminated"))
}
fn file(info: &BootInfoV1, address: u64) -> Result<RawFile> {
    let f: RawFile = load(info, address, "file")?;
    revision(f.revision, 0, "file")?;
    if f.address % 4096 != 0 {
        return Err(error("file", "alignment"));
    }
    Ok(f)
}

/// Safety: called once after the base-3 trusted loader handoff, before reclamation,
/// with exclusive kernel-owned storage. Bootstrap pointer trust is described above.
pub unsafe fn capture(info: &mut BootInfoV1) -> Result<()> {
    let h: Pair = unsafe { bootstrap(HHDM.address(), "hhdm")? };
    revision(h.revision, 0, "hhdm")?;
    if h.value < 0xffff_8000_0000_0000 || h.value % 4096 != 0 {
        return Err(error("hhdm", "virtual_address"));
    }
    info.hhdm = VirtAddr(h.value);
    let map: Array = unsafe { bootstrap(MEMORY.address(), "memory")? };
    revision(map.revision, 0, "memory")?;
    let n = count(map.count, MAX_REGIONS, "memory")?;
    if n == 0 {
        return Err(error("memory", "missing"));
    }
    numeric::<u64>(map.pointer, "memory")?;
    virtual_range(
        map.pointer,
        map.count
            .checked_mul(8)
            .ok_or(error("memory", "overflow"))?,
        "memory",
    )?;
    for i in 0..n {
        let p: u64 = unsafe { bootstrap(map.pointer + (i as u64) * 8, "memory")? };
        let r: RawRegion = unsafe { bootstrap(p, "memory")? };
        info.regions[i] = Region {
            range: Range::new(r.base, r.len, "memory")?.physical("memory")?,
            kind: r.kind,
        };
    }
    info.region_count = n;
    info.validate_memory()?;
    // Recheck all bootstrap spans using the now-validated map.
    let _: Pair = load(info, HHDM.address(), "hhdm")?;
    let _: Array = load(info, MEMORY.address(), "memory")?;
    span(info, map.pointer, map.count * 8, "memory")?;
    for i in 0..n {
        let p: u64 = load(info, map.pointer + (i as u64) * 8, "memory")?;
        let _: RawRegion = load(info, p, "memory")?;
    }
    let paging: Pair = load(info, PAGING.request.address(), "paging")?;
    revision(paging.revision, 0, "paging")?;
    if paging.value != 0 {
        return Err(error("paging", "unsupported"));
    }
    let address: Array = load(info, EXEC_ADDRESS.address(), "executable_address")?;
    revision(address.revision, 0, "executable_address")?;
    unsafe extern "C" {
        static __image_start: u8;
        static __image_end: u8;
    }
    let start = ptr::addr_of!(__image_start) as u64;
    let end = ptr::addr_of!(__image_end) as u64;
    if address.pointer != start {
        return Err(error("kernel", "virtual_base"));
    }
    info.kernel = Range::new(
        address.count,
        end.checked_sub(start).ok_or(error("kernel", "overflow"))?,
        "kernel",
    )?
    .physical("kernel")?;
    let executable = required(info, &EXEC_FILE, "executable_file")?;
    let f = file(info, executable.value)?;
    info.executable_file = from_hhdm(f.address, f.size, info.hhdm.0, "executable_file")?;
    info.executable_path = string(info, f.path, "executable_path")?;
    info.arguments = string(info, f.arguments, "arguments")?;
    if let Some(a) = array(info, &MODULES, MAX_MODULES, 2, "modules")? {
        for i in 0..a.count as usize {
            let p: u64 = load(info, a.pointer + (i as u64) * 8, "modules")?;
            let f = file(info, p)?;
            let m = Module {
                range: from_hhdm(f.address, f.size, info.hhdm.0, "module")?,
                path: string(info, f.path, "module_path")?,
                arguments: string(info, f.arguments, "module_arguments")?,
            };
            info.modules[i] = m;
        }
        info.module_count = a.count as usize;
    }
    if let Some(a) = array(info, &FRAMEBUFFERS, MAX_FRAMEBUFFERS, 1, "framebuffers")? {
        for i in 0..a.count as usize {
            let p: u64 = load(info, a.pointer + (i as u64) * 8, "framebuffers")?;
            let f: RawFramebuffer = load(info, p, "framebuffer")?;
            let len = f
                .pitch
                .checked_mul(f.height)
                .ok_or(error("framebuffer", "overflow"))?;
            info.framebuffers[i] = Framebuffer {
                address: VirtAddr(f.address),
                range: from_hhdm(f.address, len, info.hhdm.0, "framebuffer")?,
                width: f.width,
                height: f.height,
                pitch: f.pitch,
                bpp: f.bpp,
                model: f.model,
                sizes: [f.red_size, f.green_size, f.blue_size],
                shifts: [f.red_shift, f.green_shift, f.blue_shift],
            };
        }
        info.framebuffer_count = a.count as usize;
    }
    if ACPI.address() != 0 {
        info.acpi_root = Some(PhysAddr(required(info, &ACPI, "acpi")?.value));
    }
    if LOADER.address() != 0 {
        let loader: Array = load(info, LOADER.address(), "loader")?;
        revision(loader.revision, 0, "loader")?;
        info.loader_name = string(info, loader.count, "loader_name")?;
        info.loader_version = string(info, loader.pointer, "loader_version")?;
    }
    info.prerequisites = REQUIRED;
    Ok(())
}

/// Owned-input fixtures only: never replace a raw pointer or forge loader memory.
#[cfg(feature = "boot-inject")]
pub fn inject(info: &mut BootInfoV1, fixture: &str) -> Result<()> {
    match fixture {
        "normal" => {}
        "serial-only" => info.framebuffer_count = 0,
        "missing-required" => info.prerequisites &= !4,
        "unsupported-revision" => revision(1, 0, "memory")?,
        "range-overflow" => {
            info.regions[0].range = Range {
                base: u64::MAX - 1,
                len: 8,
            }
        }
        "excessive-count" => {
            count(MAX_REGIONS as u64 + 1, MAX_REGIONS, "memory")?;
        }
        "unterminated-string" => {
            info.executable_path
                .copy_cstr(&[b'x'; 256], "executable_path")?;
        }
        "framebuffer-pitch" => info.framebuffers[0].pitch = 1,
        "framebuffer-format" => info.framebuffers[0].model = 0,
        "framebuffer-range" => info.framebuffers[0].range.base += 4096,
        _ => return Err(error("fixture", "unknown")),
    }
    if fixture.starts_with("framebuffer-") && info.framebuffer_count == 0 {
        return Err(error("fixture", "missing_framebuffer"));
    }
    Ok(())
}
