//! Bounded, allocation-free static ELF admission. Trusted kernel callers only.
//! No runnable state, syscall interface or native-layout startup ABI exists here.
use crate::{
    Result, error,
    memory::PAGE,
    owned_memory::{Access, PROCESS_LIMIT, Runtime, STACK_BASE, STACK_BYTES, SpaceId, Store},
};

pub const MAX_IMAGE: usize = 1024 * 1024;
pub const MAX_LOADS: usize = 8;
pub const STARTUP: u64 = 0x7ffd_0000;
pub const RETURN: u64 = STARTUP + PAGE;
// Fail closed until P3-04 assigns process_exit and replaces this trap stub.
pub const RETURN_CODE: [u8; 2] = [0x0f, 0x0b];
pub const STARTUP_BYTES: [u8; 64] = {
    let mut b = [0; 64];
    b[0] = b'C';
    b[1] = b'N';
    b[2] = b'T';
    b[3] = b'M';
    b[4] = b'3';
    b[5] = b'R';
    b[6] = b'T';
    b[7] = b'0';
    b[12] = 64;
    b[33] = 16; // explicit LE length=64, page_bytes=4096
    b
};
fn bad(reason: &'static str) -> crate::Error {
    error("elf", reason)
}
fn span(b: &[u8], at: u64, n: u64) -> Result<&[u8]> {
    let end = at.checked_add(n).ok_or(bad("overflow"))?;
    if end > b.len() as u64 {
        return Err(bad("truncated"));
    }
    Ok(&b[at as usize..end as usize])
}
fn u16le(b: &[u8], at: u64) -> Result<u16> {
    Ok(u16::from_le_bytes(span(b, at, 2)?.try_into().unwrap()))
}
fn u32le(b: &[u8], at: u64) -> Result<u32> {
    Ok(u32::from_le_bytes(span(b, at, 4)?.try_into().unwrap()))
}
fn u64le(b: &[u8], at: u64) -> Result<u64> {
    Ok(u64::from_le_bytes(span(b, at, 8)?.try_into().unwrap()))
}
#[derive(Clone, Copy, Debug)]
pub struct Segment {
    pub address: u64,
    pub offset: u64,
    pub file_bytes: u64,
    pub memory_bytes: u64,
    pub base: u64,
    pub pages: u64,
    pub flags: u32,
}
const EMPTY: Segment = Segment {
    address: 0,
    offset: 0,
    file_bytes: 0,
    memory_bytes: 0,
    base: 0,
    pages: 0,
    flags: 0,
};
pub struct Plan {
    pub entry: u64,
    pub segments: [Segment; MAX_LOADS],
    pub count: usize,
    pub commitment: u64,
}
pub fn parse(b: &[u8]) -> Result<Plan> {
    if b.len() > MAX_IMAGE {
        return Err(bad("image_limit"));
    }
    let ident = span(b, 0, 16)?;
    // lld emits GNU OSABI for SHF_GNU_RETAIN used by #[used] fixtures.
    // This tag provides no Linux ABI: all runtime obligations are checked below.
    if ident[..7] != *b"\x7fELF\x02\x01\x01"
        || ![0, 3].contains(&ident[7])
        || ident[8..].iter().any(|b| *b != 0)
    {
        return Err(bad("identity"));
    }
    if u16le(b, 16)? != 2 || u16le(b, 18)? != 62 || u32le(b, 20)? != 1 || u32le(b, 48)? != 0 {
        return Err(bad("target"));
    }
    if u16le(b, 52)? != 64 || u16le(b, 54)? != 56 || u16le(b, 58)? != 64 {
        return Err(bad("header_size"));
    }
    let ph = u64le(b, 32)?;
    let pn = u16le(b, 56)? as u64;
    let sh = u64le(b, 40)?;
    let sn = u16le(b, 60)? as u64;
    if ph < 64 || pn == 0 || pn > 16 || sn == 0 || sn > 128 || sh < 64 || u16le(b, 62)? as u64 >= sn
    {
        return Err(bad("header_count"));
    }
    span(b, ph, pn * 56)?;
    span(b, sh, sn * 64)?;
    let mut p = Plan {
        entry: u64le(b, 24)?,
        segments: [EMPTY; MAX_LOADS],
        count: 0,
        commitment: STACK_BYTES + 2 * PAGE,
    };
    for i in 0..pn {
        let h = ph + i * 56;
        let kind = u32le(b, h)?;
        let flags = u32le(b, h + 4)?;
        if kind == 0 {
            continue;
        }
        if kind == 0x6474e551 {
            if flags != 6 {
                return Err(bad("executable_stack"));
            }
            if (1..=5).any(|n| u64le(b, h + 8 * n).is_ok_and(|v| v != 0)) {
                return Err(bad("stack_metadata"));
            }
            let alignment = u64le(b, h + 48)?;
            if alignment > PAGE || alignment != 0 && !alignment.is_power_of_two() {
                return Err(bad("stack_metadata"));
            }
            continue;
        }
        if kind != 1 {
            return Err(bad("program_kind"));
        }
        if ![4, 5, 6].contains(&flags) {
            return Err(bad("permission"));
        }
        if p.count == MAX_LOADS {
            return Err(bad("segment_limit"));
        }
        let offset = u64le(b, h + 8)?;
        let address = u64le(b, h + 16)?;
        let physical = u64le(b, h + 24)?;
        let file_bytes = u64le(b, h + 32)?;
        let memory_bytes = u64le(b, h + 40)?;
        let align = u64le(b, h + 48)?;
        if memory_bytes == 0 || file_bytes > memory_bytes {
            return Err(bad("size"));
        }
        span(b, offset, file_bytes)?;
        if align < PAGE
            || !align.is_power_of_two()
            || align > 2 * 1024 * 1024
            || address % align != offset % align
            || physical != address
        {
            return Err(bad("alignment"));
        }
        let end = address.checked_add(memory_bytes).ok_or(bad("overflow"))?;
        let rounded = end.checked_add(PAGE - 1).ok_or(bad("overflow"))? & !(PAGE - 1);
        let base = address & !(PAGE - 1);
        if base < 0x10000 || rounded > STARTUP {
            return Err(bad("user_range"));
        }
        if p.segments[..p.count]
            .iter()
            .any(|s| base < s.base + s.pages * PAGE && s.base < rounded)
        {
            return Err(bad("overlap"));
        }
        let bytes = rounded - base;
        p.commitment = p.commitment.checked_add(bytes).ok_or(bad("overflow"))?;
        if p.commitment > PROCESS_LIMIT {
            return Err(bad("commitment"));
        }
        p.segments[p.count] = Segment {
            address,
            offset,
            file_bytes,
            memory_bytes,
            base,
            pages: bytes / PAGE,
            flags,
        };
        p.count += 1;
    }
    if !p.segments[..p.count]
        .iter()
        .any(|s| s.flags == 5 && p.entry >= s.address && p.entry - s.address < s.file_bytes)
    {
        return Err(bad("entry"));
    }
    // Inspect section obligations too: a malicious non-LOAD relocation/TLS table
    // must not be silently ignored. No stripped/extended section table at rev 0.
    for i in 0..sn {
        let h = sh + i * 64;
        let kind = u32le(b, h + 4)?;
        let flags = u64le(b, h + 8)?;
        let off = u64le(b, h + 24)?;
        let size = u64le(b, h + 32)?;
        if [4, 6, 9, 11, 14, 15, 16, 19].contains(&kind) || flags & 0x400 != 0 {
            return Err(bad("section_obligation"));
        }
        if kind != 8 {
            span(b, off, size)?;
        }
        if kind == 2 {
            if u64le(b, h + 56)? != 24 || size % 24 != 0 || size > MAX_IMAGE as u64 {
                return Err(bad("symbols"));
            }
            for n in 0..size / 24 {
                let s = off + n * 24;
                if u32le(b, s)? != 0 && span(b, s + 4, 1)?[0] >> 4 != 0 && u16le(b, s + 6)? == 0 {
                    return Err(bad("undefined_symbol"));
                }
            }
        }
    }
    Ok(p)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Binding {
    UserA,
    UserB,
    Service,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Context {
    pub principal: u64,
    pub session: u64,
    pub service: bool,
    pub grant_ceiling: u64,
    pub commitment_ceiling: u64,
}
impl Binding {
    pub fn context(self) -> Context {
        match self {
            Self::UserA => Context {
                principal: 101,
                session: 1001,
                service: false,
                grant_ceiling: 1,
                commitment_ceiling: PROCESS_LIMIT,
            },
            Self::UserB => Context {
                principal: 202,
                session: 2002,
                service: false,
                grant_ceiling: 1,
                commitment_ceiling: PROCESS_LIMIT,
            },
            Self::Service => Context {
                principal: 303,
                session: 3003,
                service: true,
                grant_ceiling: 3,
                commitment_ceiling: PROCESS_LIMIT,
            },
        }
    }
}
pub struct TrustedImage<'a> {
    pub bytes: &'a [u8],
    pub digest: [u8; 32],
    pub size: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProcessId {
    pub slot: usize,
    pub generation: u64,
}
pub struct Process {
    pub space: SpaceId,
    pub context: Context,
    pub entry: u64,
    pub stack: u64,
    pub image_digest: [u8; 32],
}
struct Slot {
    generation: u64,
    process: Option<Process>,
}
pub struct Launcher<const N: usize> {
    slots: [Slot; N],
}
impl<const N: usize> Launcher<N> {
    pub const EMPTY: Self = Self {
        slots: [const {
            Slot {
                generation: 0,
                process: None,
            }
        }; N],
    };
    pub fn get(&self, id: ProcessId) -> Result<&Process> {
        self.slots
            .get(id.slot)
            .filter(|s| s.generation == id.generation)
            .and_then(|s| s.process.as_ref())
            .ok_or(error("process", "stale"))
    }
    pub fn live(&self) -> usize {
        self.slots.iter().filter(|s| s.process.is_some()).count()
    }
    /// This API belongs only to the trusted boot launcher; never wire Binding to
    /// user payload. P3-09/10 must authenticate supervisor delegation first.
    pub fn bootstrap<const S: usize, const A: usize, const M: usize, const T: usize>(
        &mut self,
        r: &mut Runtime<S, A, M, T>,
        store: &mut impl Store,
        image: TrustedImage<'_>,
        binding: Binding,
    ) -> Result<ProcessId> {
        if image.bytes.len() > MAX_IMAGE {
            return Err(bad("image_limit"));
        }
        if image.bytes.len() != image.size || sha256(image.bytes) != image.digest {
            return Err(error("image", "digest"));
        }
        let plan = parse(image.bytes)?;
        let context = binding.context();
        if plan.commitment > context.commitment_ceiling {
            return Err(error("launch", "ceiling"));
        }
        let slot = self
            .slots
            .iter()
            .position(|s| s.process.is_none() && s.generation != u64::MAX)
            .ok_or(error("process", "quota"))?;
        let space = r.create_with_stack_bytes(store, &RETURN.to_le_bytes())?;
        let result = (|| {
            for s in &plan.segments[..plan.count] {
                let a = r.allocate(store, space, s.pages * PAGE, PAGE)?.unwrap();
                r.write_staging(
                    store,
                    space,
                    a,
                    s.address - s.base,
                    span(image.bytes, s.offset, s.file_bytes)?,
                )?;
                r.map(
                    store,
                    space,
                    a,
                    s.base,
                    0,
                    s.pages * PAGE,
                    match s.flags {
                        5 => Access::Execute,
                        6 => Access::Write,
                        _ => Access::Read,
                    },
                )?;
            }
            for (address, bytes, access) in [
                (STARTUP, STARTUP_BYTES.as_slice(), Access::Read),
                (RETURN, RETURN_CODE.as_slice(), Access::Execute),
            ] {
                let a = r.allocate(store, space, PAGE, PAGE)?.unwrap();
                r.write_staging(store, space, a, 0, bytes)?;
                r.map(store, space, a, address, 0, PAGE, access)?;
            }
            Ok(())
        })();
        if let Err(e) = result {
            r.destroy(store, space)?;
            return Err(e);
        }
        // Publication: all fallible work is complete. This is STAGED, never runnable.
        self.slots[slot].generation += 1;
        self.slots[slot].process = Some(Process {
            space,
            context,
            entry: plan.entry,
            stack: STACK_BASE + STACK_BYTES - 8,
            image_digest: image.digest,
        });
        Ok(ProcessId {
            slot,
            generation: self.slots[slot].generation,
        })
    }
    pub fn destroy<const S: usize, const A: usize, const M: usize, const T: usize>(
        &mut self,
        r: &mut Runtime<S, A, M, T>,
        store: &mut impl Store,
        id: ProcessId,
    ) -> Result<()> {
        r.destroy(store, self.get(id)?.space)?;
        self.slots[id.slot].process = None;
        Ok(())
    }
    /// Later supervisor hook, no child creation API yet. Revalidate live parent,
    /// derive identity from it and narrow ceilings. Request identity is never used.
    pub fn child_context(
        &self,
        parent: ProcessId,
        principal: u64,
        session: u64,
        service: bool,
        grants: u64,
        commitment: u64,
    ) -> Result<Context> {
        let c = self.get(parent)?.context;
        if principal != c.principal
            || session != c.session
            || service != c.service
            || grants & !c.grant_ceiling != 0
            || commitment == 0
            || commitment > c.commitment_ceiling
        {
            return Err(error("launch", "authority"));
        }
        Ok(Context {
            grant_ceiling: grants,
            commitment_ceiling: commitment,
            ..c
        })
    }
}

/// SHA-256 over immutable bytes. No signing/remote package authentication claim.
pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h = [
        0x6a09e667u32,
        0xbb67ae85,
        0x3c6ef372,
        0xa54ff53a,
        0x510e527f,
        0x9b05688c,
        0x1f83d9ab,
        0x5be0cd19,
    ];
    let blocks = bytes.len() / 64 + if bytes.len() % 64 < 56 { 1 } else { 2 };
    for block in 0..blocks {
        let mut b = [0u8; 64];
        for (i, x) in b.iter_mut().enumerate() {
            let n = block * 64 + i;
            *x = if n < bytes.len() {
                bytes[n]
            } else if n == bytes.len() {
                0x80
            } else {
                0
            };
        }
        if block + 1 == blocks {
            b[56..].copy_from_slice(&((bytes.len() as u64) * 8).to_be_bytes());
        }
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes(b[i * 4..i * 4 + 4].try_into().unwrap());
        }
        for i in 16..64 {
            let x = w[i - 15];
            let y = w[i - 2];
            w[i] = w[i - 16]
                .wrapping_add(x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3))
                .wrapping_add(w[i - 7])
                .wrapping_add(y.rotate_right(17) ^ y.rotate_right(19) ^ (y >> 10));
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let t = hh
                .wrapping_add(e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25))
                .wrapping_add((e & f) ^ (!e & g))
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let u = (a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22))
                .wrapping_add((a & b) ^ (a & c) ^ (b & c));
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t);
            d = c;
            c = b;
            b = a;
            a = t.wrapping_add(u);
        }
        for (x, y) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *x = x.wrapping_add(y);
        }
    }
    let mut out = [0; 32];
    for (i, x) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&x.to_be_bytes());
    }
    out
}

#[cfg(test)]
mod tests;
