//! User-mode surface broker and software compositor; no native wire layouts.
extern crate alloc;
use self::alloc::vec::Vec;
pub const MAX_SURFACES: usize = 8;
pub const MAX_LEASES: usize = 3;
pub const BUDGET: usize = 24 * 1024 * 1024;
pub const MAX_BUFFER: usize = 8_847_360;
pub const HEADER: usize = 128;
pub type Result<T> = core::result::Result<T, u64>;
pub const INVALID: u64 = 1;
pub const AUTHORITY: u64 = 2;
pub const STALE: u64 = 3;
pub const BUSY: u64 = 4;
pub const CAPACITY: u64 = 5;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Peer {
    pub process: u64,
    pub incarnation: u64,
    pub principal: u64,
    pub session: u64,
}
#[derive(Clone, Copy)]
pub struct Message<'a> {
    pub kind: u16,
    pub sequence: u64,
    pub request: u64,
    pub resource: u64,
    pub output: u64,
    pub surface: u64,
    pub body: &'a [u8],
    pub payload: &'a [u8],
}
pub fn u32_at(b: &[u8], n: usize) -> u32 {
    u32::from_le_bytes(b[n..n + 4].try_into().unwrap())
}
pub fn u64_at(b: &[u8], n: usize) -> u64 {
    u64::from_le_bytes(b[n..n + 8].try_into().unwrap())
}
pub fn put32(b: &mut [u8], n: usize, v: u32) {
    b[n..n + 4].copy_from_slice(&v.to_le_bytes());
}
pub fn put64(b: &mut [u8], n: usize, v: u64) {
    b[n..n + 8].copy_from_slice(&v.to_le_bytes());
}
pub fn decode(b: &[u8]) -> Result<Message<'_>> {
    if b.len() < HEADER
        || b.len() > 4096
        || &b[..4] != b"CDU0"
        || b[4..8] != [0; 4]
        || u32_at(b, 8) as usize != b.len()
        || b[14..16] != [0; 2]
        || b[60..64] != [0; 4]
        || u32_at(b, 56) as usize != b.len() - 64
    {
        return Err(INVALID);
    }
    let kind = u16::from_le_bytes(b[12..14].try_into().unwrap());
    if !(1..=9).contains(&kind)
        || (kind != 3 && b.len() != HEADER)
        || u64_at(b, 16) == 0
        || u64_at(b, 24) == 0
    {
        return Err(INVALID);
    }
    Ok(Message {
        kind,
        sequence: u64_at(b, 16),
        request: u64_at(b, 24),
        resource: u64_at(b, 32),
        output: u64_at(b, 40),
        surface: u64_at(b, 48),
        body: &b[64..128],
        payload: &b[128..],
    })
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Extent {
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub bytes: usize,
}
impl Extent {
    pub fn checked(width: u32, height: u32, stride: u32, format: u32, bytes: u64) -> Result<Self> {
        let (w, h, s) = (width as usize, height as usize, stride as usize);
        if w == 0
            || h == 0
            || w > 1920
            || h > 1080
            || s < w * 4
            || s > 8192
            || s % 4 != 0
            || format != 1
            || bytes != s.checked_mul(h).ok_or(INVALID)? as u64
            || bytes > MAX_BUFFER as u64
        {
            return Err(INVALID);
        }
        Ok(Self {
            width: w,
            height: h,
            stride: s,
            bytes: bytes as usize,
        })
    }
    fn damage(self, b: &[u8]) -> Result<()> {
        let (x, y, w, h) = (u32_at(b, 40), u32_at(b, 44), u32_at(b, 48), u32_at(b, 52));
        if x.checked_add(w).ok_or(INVALID)? as usize > self.width
            || y.checked_add(h).ok_or(INVALID)? as usize > self.height
        {
            return Err(INVALID);
        }
        Ok(())
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
    Writing,
    InUse,
    Released,
}
struct Lease {
    id: u64,
    generation: u64,
    state: State,
    filled: usize,
    pixels: Vec<u8>,
}
#[derive(Clone, Copy)]
pub struct Geometry {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub visible: bool,
    pub clip: [u32; 4],
}
struct Surface {
    id: u64,
    owner: Peer,
    extent: Extent,
    revision: u64,
    next_generation: u64,
    geometry: Geometry,
    leases: [Option<Lease>; MAX_LEASES],
    active: Option<usize>,
}
pub struct Broker {
    surfaces: [Option<Surface>; MAX_SURFACES],
    next: u64,
    bytes: usize,
    frame: u64,
    presenting: bool,
}
fn zeroed(bytes: usize) -> Result<Vec<u8>> {
    let mut v = Vec::new();
    v.try_reserve_exact(bytes).map_err(|_| CAPACITY)?;
    v.resize(bytes, 0);
    Ok(v)
}
impl Broker {
    pub const fn new() -> Self {
        Self {
            surfaces: [const { None }; MAX_SURFACES],
            next: 0,
            bytes: 0,
            frame: 0,
            presenting: false,
        }
    }
    /// Kernel display-grant incarnations persist across compositor death. IDs
    /// occupy a disjoint 32-bit counter range in each incarnation, never wrapping.
    pub fn with_epoch(epoch: u64) -> Result<Self> {
        if epoch == 0 || epoch > u32::MAX as u64 {
            return Err(CAPACITY);
        }
        let mut broker = Self::new();
        broker.next = epoch << 32;
        Ok(broker)
    }
    fn next(&mut self) -> Result<u64> {
        let next = self.next.checked_add(1).ok_or(CAPACITY)?;
        if next as u32 == 0 {
            return Err(CAPACITY);
        }
        self.next = next;
        Ok(next)
    }
    fn surface(&mut self, owner: Peer, id: u64) -> Result<&mut Surface> {
        let s = self
            .surfaces
            .iter_mut()
            .flatten()
            .find(|s| s.id == id)
            .ok_or(STALE)?;
        if s.owner != owner {
            return Err(AUTHORITY);
        }
        Ok(s)
    }
    pub fn inventory(&self) -> (usize, usize, usize) {
        (
            self.surfaces.iter().flatten().count(),
            self.surfaces
                .iter()
                .flatten()
                .map(|s| s.leases.iter().flatten().count())
                .sum(),
            self.bytes,
        )
    }
    /// Endpoint assigns identity. No owner/session field in client bytes is trusted.
    pub fn request(&mut self, owner: Peer, m: Message<'_>) -> Result<u64> {
        if owner.session != 1001 || owner.process == 0 || owner.incarnation == 0 {
            return Err(AUTHORITY);
        }
        if m.resource != 1 || m.output != 1 {
            return Err(STALE);
        }
        if self.presenting {
            return Err(BUSY);
        }
        let b = m.body;
        match m.kind {
            1 => {
                if m.surface != 0 || b[..16] != [0; 16] || b[40..] != [0; 24] {
                    return Err(INVALID);
                }
                let e = Extent::checked(
                    u32_at(b, 16),
                    u32_at(b, 20),
                    u32_at(b, 24),
                    u32_at(b, 28),
                    u64_at(b, 32),
                )?;
                let index = self
                    .surfaces
                    .iter()
                    .position(Option::is_none)
                    .ok_or(CAPACITY)?;
                let id = self.next()?;
                self.surfaces[index] = Some(Surface {
                    id,
                    owner,
                    extent: e,
                    revision: 0,
                    next_generation: 0,
                    geometry: Geometry {
                        x: 0,
                        y: 0,
                        z: 0,
                        visible: true,
                        clip: [0, 0, e.width as u32, e.height as u32],
                    },
                    leases: [const { None }; MAX_LEASES],
                    active: None,
                });
                Ok(id)
            }
            2 => {
                if b != [0; 64] {
                    return Err(INVALID);
                }
                let (slot, bytes, generation) = {
                    let s = self.surface(owner, m.surface)?;
                    (
                        s.leases.iter().position(Option::is_none).ok_or(CAPACITY)?,
                        s.extent.bytes,
                        s.next_generation.checked_add(1).ok_or(CAPACITY)?,
                    )
                };
                if self.bytes.checked_add(bytes).ok_or(CAPACITY)? > BUDGET {
                    return Err(CAPACITY);
                }
                let pixels = zeroed(bytes)?;
                let id = self.next()?;
                let s = self.surface(owner, m.surface)?;
                s.next_generation = generation;
                s.leases[slot] = Some(Lease {
                    id,
                    generation,
                    state: State::Writing,
                    filled: 0,
                    pixels,
                });
                self.bytes += bytes;
                Ok(id)
            }
            3 => {
                // Chunk: lease at 0, exact sequential byte offset at 8; other fields zero.
                if b[16..] != [0; 48] || m.payload.is_empty() {
                    return Err(INVALID);
                }
                let s = self.surface(owner, m.surface)?;
                let l = s
                    .leases
                    .iter_mut()
                    .flatten()
                    .find(|l| l.id == u64_at(b, 0))
                    .ok_or(STALE)?;
                if l.state != State::Writing {
                    return Err(BUSY);
                }
                if u64_at(b, 8) != l.filled as u64 || m.payload.len() > l.pixels.len() - l.filled {
                    return Err(INVALID);
                }
                l.pixels[l.filled..l.filled + m.payload.len()].copy_from_slice(m.payload);
                l.filled += m.payload.len();
                Ok(l.filled as u64)
            }
            4 => {
                let s = self.surface(owner, m.surface)?;
                let e = Extent::checked(
                    u32_at(b, 16),
                    u32_at(b, 20),
                    u32_at(b, 24),
                    u32_at(b, 28),
                    u64_at(b, 32),
                )?;
                if e != s.extent {
                    return Err(INVALID);
                }
                e.damage(b)?;
                if u64_at(b, 8) != s.revision.checked_add(1).ok_or(STALE)? {
                    return Err(STALE);
                }
                let i = s
                    .leases
                    .iter()
                    .position(|l| l.as_ref().is_some_and(|l| l.id == u64_at(b, 0)))
                    .ok_or(STALE)?;
                let l = s.leases[i].as_ref().unwrap();
                if l.generation != u64_at(b, 56) {
                    return Err(STALE);
                }
                if l.state != State::Writing {
                    return Err(BUSY);
                }
                if l.filled != e.bytes {
                    return Err(INVALID);
                }
                // Validate premultiplication before changing active ownership.
                for row in l.pixels.chunks_exact(e.stride) {
                    for p in row[..e.width * 4].chunks_exact(4) {
                        if p[..3].iter().any(|v| *v > p[3]) {
                            return Err(INVALID);
                        }
                    }
                }
                if let Some(old) = s.active {
                    s.leases[old].as_mut().unwrap().state = State::Released;
                }
                s.leases[i].as_mut().unwrap().state = State::InUse;
                s.active = Some(i);
                s.revision = u64_at(b, 8);
                Ok(s.revision)
            }
            5 => {
                if b[..16] != [0; 16]
                    || b[32..40] != [0; 8]
                    || b[56..] != [0; 8]
                    || u32_at(b, 28) > 1
                {
                    return Err(INVALID);
                }
                let s = self.surface(owner, m.surface)?;
                s.extent.damage(b)?;
                s.geometry = Geometry {
                    x: u32_at(b, 16) as i32,
                    y: u32_at(b, 20) as i32,
                    z: u32_at(b, 24) as i32,
                    visible: u32_at(b, 28) == 1,
                    clip: [u32_at(b, 40), u32_at(b, 44), u32_at(b, 48), u32_at(b, 52)],
                };
                Ok(s.revision)
            }
            6 => {
                if b[8..] != [0; 56] {
                    return Err(INVALID);
                }
                let s = self.surface(owner, m.surface)?;
                let i = s
                    .leases
                    .iter()
                    .position(|l| l.as_ref().is_some_and(|l| l.id == u64_at(b, 0)))
                    .ok_or(STALE)?;
                if s.leases[i].as_ref().unwrap().state == State::InUse {
                    return Err(BUSY);
                }
                let bytes = s.leases[i].take().unwrap().pixels.len();
                self.bytes -= bytes;
                Ok(0)
            }
            7 => {
                if b != [0; 64] {
                    return Err(INVALID);
                }
                self.surface(owner, m.surface)?;
                let i = self
                    .surfaces
                    .iter()
                    .position(|s| s.as_ref().is_some_and(|s| s.id == m.surface))
                    .unwrap();
                let s = self.surfaces[i].take().unwrap();
                for l in s.leases.into_iter().flatten() {
                    self.bytes -= l.pixels.len();
                }
                Ok(0)
            }
            _ => Err(INVALID),
        }
    }
    /// Death is delivered through current trusted transport liveness, never client IDs.
    pub fn retain_live(&mut self, mut live: impl FnMut(Peer) -> bool) {
        for s in &mut self.surfaces {
            if s.as_ref().is_some_and(|s| !live(s.owner)) {
                for l in s.take().unwrap().leases.into_iter().flatten() {
                    self.bytes -= l.pixels.len();
                }
            }
        }
    }
    pub fn begin_frame(&mut self) -> Result<u64> {
        if self.presenting {
            return Err(BUSY);
        }
        let next = self.frame.checked_add(1).ok_or(CAPACITY)?;
        self.presenting = true;
        Ok(next)
    }
    pub fn end_frame(&mut self, frame: u64) -> Result<()> {
        if !self.presenting || frame != self.frame + 1 {
            return Err(STALE);
        }
        self.frame = frame;
        self.presenting = false;
        Ok(())
    }
    /// Full redraw clears exposure. Encoded-sRGB premultiplied source-over, rounded /255.
    pub fn render(&self, width: usize, height: usize, frame: &mut [u8]) -> Result<()> {
        if !self.presenting
            || width == 0
            || height == 0
            || width > 1920
            || height > 1080
            || frame.len() != width * height * 4
        {
            return Err(INVALID);
        }
        for p in frame.chunks_exact_mut(4) {
            p.copy_from_slice(&[12, 18, 24, 255]);
        }
        let mut order = [usize::MAX; MAX_SURFACES];
        let mut n = 0;
        for (i, s) in self.surfaces.iter().enumerate() {
            if s.as_ref()
                .is_some_and(|s| s.geometry.visible && s.active.is_some())
            {
                order[n] = i;
                n += 1;
            }
        }
        order[..n].sort_unstable_by_key(|i| {
            let s = self.surfaces[*i].as_ref().unwrap();
            (s.geometry.z, s.id)
        });
        for i in &order[..n] {
            let s = self.surfaces[*i].as_ref().unwrap();
            let g = s.geometry;
            let l = s.leases[s.active.unwrap()].as_ref().unwrap();
            for y in g.clip[1]..g.clip[1] + g.clip[3] {
                let dy = i64::from(g.y) + i64::from(y);
                if dy < 0 || dy >= height as i64 {
                    continue;
                }
                for x in g.clip[0]..g.clip[0] + g.clip[2] {
                    let dx = i64::from(g.x) + i64::from(x);
                    if dx < 0 || dx >= width as i64 {
                        continue;
                    }
                    let a = y as usize * s.extent.stride + x as usize * 4;
                    let b = (dy as usize * width + dx as usize) * 4;
                    let alpha = l.pixels[a + 3] as u32;
                    for c in 0..3 {
                        frame[b + c] = (l.pixels[a + c] as u32
                            + (frame[b + c] as u32 * (255 - alpha) + 127) / 255)
                            as u8;
                    }
                }
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    const A: Peer = Peer {
        process: 4,
        incarnation: 1,
        principal: 403,
        session: 1001,
    };
    const B: Peer = Peer {
        process: 5,
        incarnation: 1,
        principal: 404,
        session: 1001,
    };
    fn message(kind: u8, surface: u64) -> [u8; 128] {
        // Independently placed literal envelope, rather than production encoder.
        let mut b = [0; 128];
        b[..16].copy_from_slice(&[67, 68, 85, 48, 0, 0, 0, 0, 128, 0, 0, 0, kind, 0, 0, 0]);
        b[16] = 1;
        b[24] = 1;
        b[32] = 1;
        b[40] = 1;
        b[48..56].copy_from_slice(&surface.to_le_bytes());
        b[56] = 64;
        b
    }
    fn call(b: &mut Broker, p: Peer, data: &[u8]) -> Result<u64> {
        b.request(p, decode(data)?)
    }
    fn create(b: &mut Broker, p: Peer, w: u32, h: u32, stride: u32) -> u64 {
        let mut m = message(1, 0);
        put32(&mut m, 80, w);
        put32(&mut m, 84, h);
        put32(&mut m, 88, stride);
        put32(&mut m, 92, 1);
        put64(&mut m, 96, stride as u64 * h as u64);
        call(b, p, &m).unwrap()
    }
    fn acquire(b: &mut Broker, p: Peer, s: u64) -> u64 {
        call(b, p, &message(2, s)).unwrap()
    }
    fn write(b: &mut Broker, p: Peer, s: u64, l: u64, data: &[u8]) -> Result<u64> {
        let mut m = message(3, s).to_vec();
        put64(&mut m, 64, l);
        m.extend_from_slice(data);
        let n = m.len();
        put32(&mut m, 8, n as u32);
        put32(&mut m, 56, n as u32 - 64);
        call(b, p, &m)
    }
    fn commit(
        b: &mut Broker,
        p: Peer,
        s: u64,
        l: u64,
        g: u64,
        revision: u64,
        w: u32,
        h: u32,
        stride: u32,
    ) -> Result<u64> {
        let mut m = message(4, s);
        put64(&mut m, 64, l);
        put64(&mut m, 72, revision);
        put32(&mut m, 80, w);
        put32(&mut m, 84, h);
        put32(&mut m, 88, stride);
        put32(&mut m, 92, 1);
        put64(&mut m, 96, stride as u64 * h as u64);
        put32(&mut m, 112, w);
        put32(&mut m, 116, h);
        put64(&mut m, 120, g);
        call(b, p, &m)
    }
    fn frame(b: &mut Broker) -> [u8; 64] {
        let n = b.begin_frame().unwrap();
        let mut f = [0; 64];
        b.render(4, 4, &mut f).unwrap();
        b.end_frame(n).unwrap();
        f
    }
    #[test]
    fn compositor_epochs_cannot_reuse_old_surface_or_lease_ids() {
        let mut old = Broker::with_epoch(1).unwrap();
        let old_surface = create(&mut old, A, 1, 1, 4);
        let old_lease = acquire(&mut old, A, old_surface);
        let mut new = Broker::with_epoch(2).unwrap();
        let surface = create(&mut new, A, 1, 1, 4);
        let lease = acquire(&mut new, A, surface);
        assert_ne!(old_surface, surface);
        assert_ne!(old_lease, lease);
        assert_eq!(call(&mut new, A, &message(2, old_surface)), Err(STALE));
        assert_eq!(write(&mut new, A, surface, old_lease, &[0; 4]), Err(STALE));
        assert!(Broker::with_epoch(0).is_err());
        assert!(Broker::with_epoch(u32::MAX as u64 + 1).is_err());
        new.next = (2 << 32) | u32::MAX as u64;
        let before = new.inventory();
        assert_eq!(call(&mut new, A, &message(2, surface)), Err(CAPACITY));
        assert_eq!(new.inventory(), before);
    }
    #[test]
    fn literal_codec_truncations_reserved_lengths_and_extents() {
        let m = message(2, 9);
        assert_eq!(decode(&m).unwrap().surface, 9);
        for n in 0..128 {
            assert!(decode(&m[..n]).is_err());
        }
        for at in [0, 4, 6, 8, 12, 14, 16, 24, 56, 60] {
            let mut bad = m;
            bad[at] ^= 0xff;
            assert!(decode(&bad).is_err() || at == 16 || at == 24);
        }
        for (w, h, s, f, n) in [
            (0, 1, 4, 1, 4),
            (1, 0, 4, 1, 0),
            (1921, 1, 7684, 1, 7684),
            (1, 1, 3, 1, 3),
            (1, 1, 5, 1, 5),
            (1, 1, 4, 2, 4),
            (1, 1, 4, 1, u64::MAX),
            (u32::MAX, 1, u32::MAX, 1, 4),
        ] {
            assert!(Extent::checked(w, h, s, f, n).is_err());
        }
    }
    #[test]
    fn snapshot_alias_exclusion_authority_release_and_death() {
        let mut b = Broker::new();
        let s = create(&mut b, A, 2, 2, 12);
        let l = acquire(&mut b, A, s);
        let initial = frame(&mut b);
        assert_eq!(call(&mut b, B, &message(2, s)), Err(AUTHORITY));
        assert_eq!(
            call(&mut b, Peer { session: 2002, ..A }, &message(2, s)),
            Err(AUTHORITY)
        );
        assert_eq!(commit(&mut b, A, s, l, 1, 1, 2, 2, 12), Err(INVALID));
        assert_eq!(frame(&mut b), initial);
        let mut pixels = [0; 24];
        for y in 0..2 {
            for x in 0..2 {
                pixels[y * 12 + x * 4..y * 12 + x * 4 + 4].copy_from_slice(&[100, 40, 20, 128]);
            }
        }
        pixels[8..12].fill(0xfe);
        pixels[20..24].fill(0xfe);
        write(&mut b, A, s, l, &pixels).unwrap();
        pixels.fill(0);
        commit(&mut b, A, s, l, 1, 1, 2, 2, 12).unwrap();
        let f = frame(&mut b);
        assert_eq!(&f[..4], &[106, 49, 32, 255]);
        assert_eq!(&f[8..12], &[12, 18, 24, 255]);
        assert_eq!(write(&mut b, A, s, l, &[0; 4]), Err(BUSY));
        let mut rel = message(6, s);
        put64(&mut rel, 64, l);
        assert_eq!(call(&mut b, A, &rel), Err(BUSY));
        assert_eq!(commit(&mut b, A, s, l, 1, 2, 2, 2, 12), Err(BUSY));
        assert_eq!(frame(&mut b), f);
        let n = b.begin_frame().unwrap();
        assert_eq!(call(&mut b, A, &message(7, s)), Err(BUSY));
        assert_eq!(b.end_frame(n + 1), Err(STALE));
        b.end_frame(n).unwrap();
        b.retain_live(|p| p != A);
        assert_eq!(b.inventory(), (0, 0, 0));
        assert_eq!(frame(&mut b), initial);
        assert_eq!(call(&mut b, A, &rel), Err(STALE));
    }
    #[test]
    fn full_redraw_repairs_move_hide_remove_z_clip_and_alpha() {
        let mut b = Broker::new();
        let s = create(&mut b, A, 2, 2, 8);
        let l = acquire(&mut b, A, s);
        write(
            &mut b,
            A,
            s,
            l,
            &[
                200, 0, 0, 255, 200, 0, 0, 255, 200, 0, 0, 255, 200, 0, 0, 255,
            ],
        )
        .unwrap();
        commit(&mut b, A, s, l, 1, 1, 2, 2, 8).unwrap();
        let t = create(&mut b, B, 2, 2, 8);
        let q = acquire(&mut b, B, t);
        write(
            &mut b,
            B,
            t,
            q,
            &[
                0, 100, 0, 128, 0, 100, 0, 128, 0, 100, 0, 128, 0, 100, 0, 128,
            ],
        )
        .unwrap();
        commit(&mut b, B, t, q, 1, 1, 2, 2, 8).unwrap();
        assert_eq!(&frame(&mut b)[..4], &[100, 100, 0, 255]);
        let mut m = message(5, t);
        put32(&mut m, 80, 1);
        put32(&mut m, 84, 1);
        put32(&mut m, 92, 1);
        put32(&mut m, 112, 2);
        put32(&mut m, 116, 2);
        call(&mut b, B, &m).unwrap();
        let f = frame(&mut b);
        assert_eq!(&f[..4], &[200, 0, 0, 255]);
        assert_eq!(&f[20..24], &[100, 100, 0, 255]);
        put32(&mut m, 92, 0);
        call(&mut b, B, &m).unwrap();
        assert_eq!(&frame(&mut b)[20..24], &[200, 0, 0, 255]);
        put32(&mut m, 92, 1);
        put32(&mut m, 80, (-1i32) as u32);
        put32(&mut m, 104, 1);
        put32(&mut m, 112, 1);
        call(&mut b, B, &m).unwrap();
        let f = frame(&mut b);
        assert_eq!(&f[16..20], &[100, 100, 0, 255]);
        assert_eq!(&f[20..24], &[200, 0, 0, 255]);
        call(&mut b, B, &message(7, t)).unwrap();
        assert_eq!(&frame(&mut b)[16..20], &[200, 0, 0, 255]);
        b.retain_live(|_| false);
        assert_eq!(b.inventory(), (0, 0, 0));
    }
    #[test]
    fn lease_and_global_byte_admission_rollback_and_generation() {
        let mut b = Broker::new();
        let s = create(&mut b, A, 1920, 1080, 8192);
        acquire(&mut b, A, s);
        acquire(&mut b, A, s);
        let before = b.inventory();
        assert_eq!(call(&mut b, A, &message(2, s)), Err(CAPACITY));
        assert_eq!(b.inventory(), before);
        b.retain_live(|_| false);
        assert_eq!(b.inventory(), (0, 0, 0));
        let s = create(&mut b, A, 1, 1, 4);
        let a = acquire(&mut b, A, s);
        let c = acquire(&mut b, A, s);
        acquire(&mut b, A, s);
        assert_eq!(call(&mut b, A, &message(2, s)), Err(CAPACITY));
        write(&mut b, A, s, a, &[1, 2, 3, 255]).unwrap();
        commit(&mut b, A, s, a, 1, 1, 1, 1, 4).unwrap();
        write(&mut b, A, s, c, &[3, 2, 1, 255]).unwrap();
        assert_eq!(commit(&mut b, A, s, c, 1, 2, 1, 1, 4), Err(STALE));
        commit(&mut b, A, s, c, 2, 2, 1, 1, 4).unwrap();
        let mut rel = message(6, s);
        put64(&mut rel, 64, a);
        call(&mut b, A, &rel).unwrap();
        assert_eq!(call(&mut b, A, &rel), Err(STALE));
        b.retain_live(|_| false);
        for _ in 0..MAX_SURFACES {
            create(&mut b, A, 1, 1, 4);
        }
        let mut extra = message(1, 0);
        put32(&mut extra, 80, 1);
        put32(&mut extra, 84, 1);
        put32(&mut extra, 88, 4);
        put32(&mut extra, 92, 1);
        put64(&mut extra, 96, 4);
        assert_eq!(call(&mut b, A, &extra), Err(CAPACITY));
    }
}
