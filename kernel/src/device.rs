//! Bounded trusted device broker primitives. No user ABI and no DMA confinement.
//! Constructors are used by trusted discovery/launch code, never from device IDs
//! supplied by an app. Hardware completion is observed by the architecture adapter.
use crate::{Range, Result, error};

pub const MAX_DEVICES: usize = 3;
pub const QUEUE_SIZE: usize = 8;
pub const QUARANTINE_BYTES: u64 = 16 * 1024 * 1024;
pub const MAP: u8 = 1;
pub const IRQ: u8 = 2;
pub const DMA: u8 = 4;

/// PCI BARs can be above 4 GiB. Leave the legacy APIC/HPET helper's bound
/// unchanged; new pages retain checked 52-bit physical and reservation policy.
pub fn mmio_page(regions: &[crate::Region], base: u64) -> Result<Resource> {
    let resource = Resource::new(base, 4096, Kind::Mmio)?;
    if regions
        .iter()
        .any(|r| r.kind != 1 && r.range.overlaps(resource.range))
    {
        return Err(error("device_range", "reserved"));
    }
    Ok(resource)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Mmio,
    Port,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Resource {
    range: Range,
    kind: Kind,
}
impl Resource {
    pub fn new(base: u64, len: u64, kind: Kind) -> Result<Self> {
        let range = Range::new(base, len, "device_range")?.physical("device_range")?;
        if base == 0
            || match kind {
                Kind::Port => range.end() > 65536,
                Kind::Mmio => base % 4096 != 0 || len % 4096 != 0,
            }
        {
            return Err(error("device_range", "alignment_or_limit"));
        }
        Ok(Self { range, kind })
    }
    pub fn range(self) -> Range {
        self.range
    }
    pub fn access(self, offset: u64, width: u64) -> Result<u64> {
        if ![1, 2, 4].contains(&width)
            || offset % width != 0
            || offset.checked_add(width).is_none_or(|v| v > self.range.len)
        {
            return Err(error("device_range", "access"));
        }
        Ok(self.range.base + offset)
    }
}

/// A trusted process incarnation, not a principal alone (same-user apps differ).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Owner {
    pub process: u64,
    pub generation: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grant {
    slot: usize,
    epoch: u64,
}
#[derive(Clone, Copy)]
struct Device {
    resource: Resource,
    owner: Owner,
    rights: u8,
    epoch: u64,
    masked: bool,
    pending: bool,
    sequence: u64,
}
pub struct Broker {
    devices: [Option<Device>; MAX_DEVICES],
    next_epoch: u64,
}
impl Broker {
    pub const EMPTY: Self = Self {
        devices: [None; MAX_DEVICES],
        next_epoch: 1,
    };
    /// Only trusted bootstrap selects the owner, rights and validated resource.
    pub fn grant(&mut self, owner: Owner, resource: Resource, rights: u8) -> Result<Grant> {
        if owner.process == 0
            || owner.generation == 0
            || rights == 0
            || rights & !(MAP | IRQ | DMA) != 0
        {
            return Err(error("device_grant", "authority"));
        }
        if self
            .devices
            .iter()
            .flatten()
            .any(|d| d.resource.kind == resource.kind && d.resource.range.overlaps(resource.range))
        {
            return Err(error("device_grant", "overlap"));
        }
        let slot = self
            .devices
            .iter()
            .position(Option::is_none)
            .ok_or(error("device_grant", "capacity"))?;
        let epoch = self.next_epoch;
        self.next_epoch = epoch.checked_add(1).ok_or(error("device_grant", "epoch"))?;
        self.devices[slot] = Some(Device {
            resource,
            owner,
            rights,
            epoch,
            masked: true,
            pending: false,
            sequence: 0,
        });
        Ok(Grant { slot, epoch })
    }
    fn get(&mut self, owner: Owner, grant: Grant, rights: u8) -> Result<&mut Device> {
        self.devices
            .get_mut(grant.slot)
            .and_then(Option::as_mut)
            .filter(|d| d.owner == owner && d.epoch == grant.epoch && d.rights & rights == rights)
            .ok_or(error("device_grant", "denied"))
    }
    /// Mediated access only; this never constructs a user page-table mapping.
    pub fn map(&mut self, owner: Owner, grant: Grant, offset: u64, width: u64) -> Result<u64> {
        self.get(owner, grant, MAP)?.resource.access(offset, width)
    }
    pub fn mask(&mut self, owner: Owner, grant: Grant, masked: bool) -> Result<()> {
        self.get(owner, grant, IRQ)?.masked = masked;
        Ok(())
    }
    /// IRQ-side notification is a coalescing latch; parsing remains outside IRQ.
    pub fn notify(&mut self, grant: Grant) -> Result<()> {
        let d = self
            .devices
            .get_mut(grant.slot)
            .and_then(Option::as_mut)
            .filter(|d| d.epoch == grant.epoch)
            .ok_or(error("device_irq", "stale"))?;
        if !d.pending {
            d.sequence = d
                .sequence
                .checked_add(1)
                .ok_or(error("device_irq", "sequence"))?;
            d.pending = true;
        }
        Ok(())
    }
    pub fn wait(&mut self, owner: Owner, grant: Grant) -> Result<Option<u64>> {
        let d = self.get(owner, grant, IRQ)?;
        Ok((d.pending && !d.masked).then_some(d.sequence))
    }
    pub fn ack(&mut self, owner: Owner, grant: Grant, sequence: u64) -> Result<()> {
        let d = self.get(owner, grant, IRQ)?;
        if !d.pending || d.sequence != sequence {
            return Err(error("device_irq", "sequence"));
        }
        d.pending = false;
        Ok(())
    }
    /// Authority revocation does NOT free DMA. The adapter must stop/reset first.
    pub fn revoke(&mut self, owner: Owner, grant: Grant) -> Result<()> {
        self.get(owner, grant, 0)?;
        self.devices[grant.slot] = None;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    ToDevice,
    FromDevice,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Descriptor {
    pub address: u64,
    pub len: u32,
    pub flags: u16,
    pub next: u16,
}
/// Validate a bounded split-ring chain against a single exclusive admitted run.
/// Indirect descriptors, event-index and packed rings are deliberately absent.
pub fn chain(desc: &[Descriptor], head: u16, backing: Range, direction: Direction) -> Result<u64> {
    let backing = Range::new(backing.base, backing.len, "dma")?.physical("dma")?;
    if backing.base == 0 || desc.is_empty() || desc.len() > QUEUE_SIZE {
        return Err(error("dma", "capacity"));
    }
    let mut seen = 0u16;
    let mut at = head as usize;
    let mut total = 0u64;
    loop {
        let d = desc.get(at).ok_or(error("dma", "index"))?;
        if seen & (1 << at) != 0
            || d.flags & !3 != 0
            || (d.flags & 2 != 0) != (direction == Direction::FromDevice)
        {
            return Err(error("dma", "flags_or_cycle"));
        }
        let r = Range::new(d.address, u64::from(d.len), "dma")?.physical("dma")?;
        if !backing.contains(r) {
            return Err(error("dma", "bounds"));
        }
        for (i, old) in desc.iter().enumerate() {
            if seen & (1 << i) != 0
                && r.overlaps(Range {
                    base: old.address,
                    len: u64::from(old.len),
                })
            {
                return Err(error("dma", "overlap"));
            }
        }
        seen |= 1 << at;
        total = total
            .checked_add(u64::from(d.len))
            .ok_or(error("dma", "overflow"))?;
        if d.flags & 1 == 0 {
            return Ok(total);
        }
        at = d.next as usize;
    }
}

/// Offered unknown bits are optional: negotiate only the implemented subset.
/// Required bits are trusted driver requirements, not every offered device bit.
pub fn features(offered: u64, required: u64) -> Result<u64> {
    const SUPPORTED: u64 = 1 << 32; // VIRTIO_F_VERSION_1; coherent x86 DMA only
    if required & !SUPPORTED != 0 || offered & (required | SUPPORTED) != required | SUPPORTED {
        return Err(error("virtio", "features"));
    }
    Ok(SUPPORTED)
}
pub fn queue_size(maximum: u16) -> Result<u16> {
    if maximum < QUEUE_SIZE as u16 || !maximum.is_power_of_two() {
        return Err(error("virtio", "queue_size"));
    }
    Ok(QUEUE_SIZE as u16)
}
/// Status-queue probe payload: a generic SYN_REPORT no-op, or NumLock feedback
/// only after the device advertises that exact LED code. No event parser ABI.
pub fn feedback(led_supported: bool, request_led: bool) -> Result<[u8; 8]> {
    if request_led {
        if !led_supported {
            return Err(error("virtio_input", "unsupported_led"));
        }
        Ok([17, 0, 0, 0, 1, 0, 0, 0])
    } else {
        Ok([0; 8])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DmaState {
    Cpu,
    Device,
    Stopping,
    Quarantined,
    Reset,
}
pub struct Dma {
    range: Range,
    state: DmaState,
    outstanding: u16,
    completed: u16,
}
/// Admission reserves worst-case retention before enabling any device. There
/// can be at most three retained devices / 16 MiB including rings and payloads.
pub struct Retention {
    bytes: u64,
    devices: usize,
    ranges: [Option<Range>; MAX_DEVICES],
}
impl Retention {
    pub const EMPTY: Self = Self {
        bytes: 0,
        devices: 0,
        ranges: [None; MAX_DEVICES],
    };
    pub fn admit(&mut self, base: u64, bytes: u64, alignment: u64) -> Result<Dma> {
        let range = Range::new(base, bytes, "dma")?.physical("dma")?;
        if alignment < 4096
            || !alignment.is_power_of_two()
            || base == 0
            || base % alignment != 0
            || bytes % 4096 != 0
            || self.devices == MAX_DEVICES
            || bytes > QUARANTINE_BYTES - self.bytes
            || self.ranges.iter().flatten().any(|r| r.overlaps(range))
        {
            return Err(error("dma", "admission"));
        }
        self.bytes += bytes;
        self.devices += 1;
        let slot = self.ranges.iter().position(Option::is_none).unwrap();
        self.ranges[slot] = Some(range);
        Ok(Dma {
            range,
            state: DmaState::Cpu,
            outstanding: 0,
            completed: 0,
        })
    }
    pub fn inventory(&self) -> (usize, u64) {
        (self.devices, self.bytes)
    }
    /// Token consumption prevents double release. Errors return ownership intact.
    pub fn release(&mut self, dma: Dma) -> core::result::Result<Range, Dma> {
        if !matches!(dma.state, DmaState::Cpu | DmaState::Reset) {
            return Err(dma);
        }
        let Some(slot) = self.ranges.iter().position(|r| *r == Some(dma.range)) else {
            return Err(dma);
        };
        self.ranges[slot] = None;
        self.devices -= 1;
        self.bytes -= dma.range.len;
        Ok(dma.range)
    }
}
impl Dma {
    pub fn range(&self) -> Range {
        self.range
    }
    pub fn state(&self) -> DmaState {
        self.state
    }
    /// Called before publication, which must use a release fence in the adapter.
    pub fn submit(&mut self, head: u16) -> Result<()> {
        if head as usize >= QUEUE_SIZE
            || !matches!(self.state, DmaState::Cpu | DmaState::Device)
            || (self.outstanding | self.completed) & (1 << head) != 0
        {
            return Err(error("dma", "submission"));
        }
        self.state = DmaState::Device;
        self.outstanding |= 1 << head;
        Ok(())
    }
    /// A checked used entry transfers that payload, never the live ring itself.
    pub fn complete(&mut self, head: u32, len: u32, capacity: u32) -> Result<()> {
        if head >= QUEUE_SIZE as u32
            || len > capacity
            || !matches!(self.state, DmaState::Device | DmaState::Stopping)
            || self.outstanding & (1 << head) == 0
        {
            return Err(error("dma", "completion"));
        }
        self.outstanding &= !(1 << head);
        self.completed |= 1 << head;
        Ok(())
    }
    /// Cancellation/death stops admission. It cannot retract a published buffer.
    pub fn stop(&mut self) {
        if self.state != DmaState::Reset {
            self.state = DmaState::Stopping;
        }
    }
    pub fn timeout(&mut self) {
        if self.state != DmaState::Reset {
            self.state = DmaState::Quarantined;
        }
    }
    /// Trusted adapter only: call after observing device_status == 0, draining
    /// ISR and masking the routed IRQ. No user-supplied reset boolean is accepted.
    pub fn reset_observed(&mut self) -> Result<()> {
        if !matches!(self.state, DmaState::Stopping | DmaState::Quarantined) {
            return Err(error("dma", "reset_order"));
        }
        self.state = DmaState::Reset;
        self.outstanding = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const OWNER: Owner = Owner {
        process: 402,
        generation: 1,
    };
    fn resource(base: u64) -> Resource {
        Resource::new(base, 4096, Kind::Mmio).unwrap()
    }
    #[test]
    fn authority_is_incarnation_bound_and_overlap_denied() {
        let mut b = Broker::EMPTY;
        let g = b.grant(OWNER, resource(0xf0000000), MAP | IRQ).unwrap();
        assert_eq!(b.map(OWNER, g, 4092, 4).unwrap(), 0xf0000ffc);
        assert!(b.grant(OWNER, resource(0xf0000000), MAP).is_err());
        for owner in [
            Owner {
                process: 403,
                ..OWNER
            },
            Owner {
                generation: 2,
                ..OWNER
            },
        ] {
            assert!(b.map(owner, g, 0, 4).is_err());
            assert!(b.ack(owner, g, 1).is_err());
        }
        assert!(b.map(OWNER, Grant { epoch: 2, ..g }, 0, 4).is_err());
        assert!(b.map(OWNER, g, 4096, 4).is_err());
        assert!(b.map(OWNER, g, 1, 4).is_err());
        assert!(b.map(OWNER, g, u64::MAX, 1).is_err());
        b.revoke(OWNER, g).unwrap();
        let new = b.grant(OWNER, resource(0xf0000000), IRQ).unwrap();
        assert_ne!(g, new);
        assert!(b.map(OWNER, new, 0, 4).is_err());
        assert!(b.notify(g).is_err());
    }
    #[test]
    fn irq_latch_mask_duplicate_and_cancel() {
        let mut b = Broker::EMPTY;
        let g = b.grant(OWNER, resource(0xf0000000), IRQ).unwrap();
        b.notify(g).unwrap();
        b.notify(g).unwrap();
        assert_eq!(b.wait(OWNER, g).unwrap(), None);
        b.mask(OWNER, g, false).unwrap();
        assert_eq!(b.wait(OWNER, g).unwrap(), Some(1));
        assert!(b.ack(OWNER, g, 0).is_err());
        b.ack(OWNER, g, 1).unwrap();
        assert!(b.ack(OWNER, g, 1).is_err());
        b.notify(g).unwrap();
        assert_eq!(b.wait(OWNER, g).unwrap(), Some(2));
        b.revoke(OWNER, g).unwrap();
        assert!(b.wait(OWNER, g).is_err());
    }
    #[test]
    fn ranges_features_and_queue_limits() {
        for (base, len, kind) in [
            (0, 4096, Kind::Mmio),
            (1, 4096, Kind::Mmio),
            ((1u64 << 52) - 4096, 8192, Kind::Mmio),
            (65535, 2, Kind::Port),
            (u64::MAX, 4, Kind::Port),
        ] {
            assert!(Resource::new(base, len, kind).is_err());
        }
        assert_eq!(features(u64::MAX, 1 << 32).unwrap(), 1 << 32);
        assert!(features(0, 1 << 32).is_err());
        assert!(features(u64::MAX, 1 << 33).is_err());
        for n in [0, 1, 7, 9, 65535] {
            assert!(queue_size(n).is_err());
        }
        assert_eq!(queue_size(256).unwrap(), 8);
        assert!(feedback(false, true).is_err());
        assert_eq!(feedback(false, false).unwrap(), [0; 8]);
        assert_eq!(feedback(true, true).unwrap(), [17, 0, 0, 0, 1, 0, 0, 0]);
        assert!(mmio_page(&[], 0x800000000).is_ok());
        let ram = [crate::Region {
            range: Range {
                base: 0x800000000,
                len: 4096,
            },
            kind: 0,
        }];
        assert!(mmio_page(&ram, 0x800000000).is_err());
    }
    #[test]
    fn chain_bounds_cycle_flags_and_overlap() {
        let backing = Range {
            base: 4096,
            len: 4096,
        };
        let d = Descriptor {
            address: 4096,
            len: 8,
            flags: 2,
            next: 0,
        };
        assert_eq!(chain(&[d], 0, backing, Direction::FromDevice).unwrap(), 8);
        for bad in [
            Descriptor { len: 0, ..d },
            Descriptor {
                address: u64::MAX,
                ..d
            },
            Descriptor { len: 4097, ..d },
            Descriptor { flags: 4, ..d },
            Descriptor { flags: 3, ..d },
            Descriptor { flags: 0, ..d },
        ] {
            assert!(chain(&[bad], 0, backing, Direction::FromDevice).is_err());
        }
        assert!(
            chain(
                &[
                    Descriptor {
                        flags: 3,
                        next: 1,
                        ..d
                    },
                    d
                ],
                0,
                backing,
                Direction::FromDevice
            )
            .is_err()
        );
    }
    #[test]
    fn no_reclaim_before_reset_even_after_used_completion() {
        let mut r = Retention::EMPTY;
        let mut d = r.admit(4096, 16384, 4096).unwrap();
        for head in 0..8 {
            d.submit(head).unwrap();
        }
        assert!(d.submit(8).is_err());
        assert!(d.complete(8, 8, 8).is_err());
        assert!(d.complete(0, 9, 8).is_err());
        d.complete(0, 8, 8).unwrap();
        assert!(d.complete(0, 8, 8).is_err());
        let mut d = match r.release(d) {
            Err(d) => d,
            Ok(_) => panic!("live DMA freed"),
        };
        d.stop();
        assert!(d.submit(1).is_err());
        d.timeout();
        let mut d = match r.release(d) {
            Err(d) => d,
            Ok(_) => panic!("quarantine freed"),
        };
        assert_eq!(r.inventory(), (1, 16384));
        d.reset_observed().unwrap();
        assert!(r.release(d).is_ok());
        assert_eq!(r.inventory(), (0, 0));
    }
    #[test]
    fn quarantine_reserved_before_dma() {
        let mut r = Retention::EMPTY;
        let d = r.admit(4096, QUARANTINE_BYTES, 4096).unwrap();
        assert!(r.admit(0x2000000, 4096, 4096).is_err());
        assert!(r.release(d).is_ok());
        let mut held = [const { None }; 3];
        for (n, slot) in held.iter_mut().enumerate() {
            *slot = Some(r.admit(4096 + n as u64 * 4096, 4096, 4096).unwrap());
        }
        assert!(r.admit(0x10000, 4096, 4096).is_err());
        for slot in &mut held {
            assert!(r.release(slot.take().unwrap()).is_ok());
        }
        assert_eq!(r.inventory(), (0, 0));
    }
}
