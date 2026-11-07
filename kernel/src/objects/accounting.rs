//! Read-only physical backing classification. Attribution can overlap; the
//! three global classes below are mutually exclusive and sum to backing bytes.
use super::*;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackingUsage {
    pub private_bytes: u64,
    pub shared_bytes: u64,
    pub queued_only_bytes: u64,
    pub owner_attribution: [u64; OWNERS],
    pub memory_handles: usize,
    pub memory_maps: usize,
    pub queued_references: usize,
    pub timers: usize,
    pub events: usize,
}
impl Registry {
    pub fn referenced_bytes(&self, space: SpaceId) -> Result<u64> {
        Ok(self.backing_usage().owner_attribution[self.table(space)?])
    }
    pub fn backing_usage(&self) -> BackingUsage {
        let mut usage = BackingUsage {
            private_bytes: 0,
            shared_bytes: 0,
            queued_only_bytes: 0,
            owner_attribution: [0; OWNERS],
            memory_handles: 0,
            memory_maps: 0,
            queued_references: 0,
            timers: 0,
            events: 0,
        };
        // Traverse each reference once, rather than rescanning every owner and
        // channel queue for each memory object. This remains a fresh read-only
        // snapshot: no cached counters or mutation-path invalidation policy.
        const { assert!(OWNERS <= u16::BITS as usize) };
        let mut owners = [0u16; OBJECTS];
        let mut queued = [0usize; OBJECTS];
        let memory_slot = |id: Id| {
            self.objects
                .get(id.slot)
                .filter(|object| {
                    object.generation == id.generation
                        && matches!(object.body, Some(Body::Memory { .. }))
                })
                .map(|_| id.slot)
        };
        for (owner, table) in self.tables.iter().enumerate() {
            for handle in table.handles.iter().flatten() {
                if let Some(slot) = memory_slot(handle.object) {
                    usage.memory_handles += 1;
                    owners[slot] |= 1 << owner;
                }
            }
            for mapping in table.mappings.iter().flatten() {
                if let Some(slot) = memory_slot(mapping.object) {
                    usage.memory_maps += 1;
                    owners[slot] |= 1 << owner;
                }
            }
        }
        self.visit_queued_references(|id| {
            if let Some(slot) = memory_slot(id) {
                queued[slot] += 1;
            }
        });
        for (slot, object) in self.objects.iter().enumerate() {
            usage.timers += usize::from(matches!(object.body, Some(Body::Timer(_))));
            usage.events += usize::from(matches!(object.body, Some(Body::Event(_))));
            let Some(Body::Memory { bytes, .. }) = object.body else {
                continue;
            };
            for owner in 0..OWNERS {
                if owners[slot] & (1 << owner) != 0 {
                    usage.owner_attribution[owner] += bytes;
                }
            }
            // Object.handles includes queue-owned references. Ordinary handle
            // count above excludes those committed transfer holds.
            usage.queued_references += queued[slot];
            match owners[slot].count_ones() {
                0 => {
                    assert!(queued[slot] > 0);
                    usage.queued_only_bytes += bytes;
                }
                1 => usage.private_bytes += bytes,
                _ => usage.shared_bytes += bytes,
            }
        }
        usage
    }
}
