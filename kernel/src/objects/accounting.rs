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
        for (slot, object) in self.objects.iter().enumerate() {
            usage.timers += usize::from(matches!(object.body, Some(Body::Timer(_))));
            usage.events += usize::from(matches!(object.body, Some(Body::Event(_))));
            let Some(Body::Memory { bytes, .. }) = object.body else {
                continue;
            };
            let mut owners = 0;
            let id = Id {
                slot,
                generation: object.generation,
            };
            for (owner, table) in self.tables.iter().enumerate() {
                let handles = table
                    .handles
                    .iter()
                    .flatten()
                    .filter(|h| h.object == id)
                    .count();
                let maps = table
                    .mappings
                    .iter()
                    .flatten()
                    .filter(|m| m.object == id)
                    .count();
                usage.memory_handles += handles;
                usage.memory_maps += maps;
                if handles + maps != 0 {
                    owners += 1;
                    usage.owner_attribution[owner] += bytes;
                }
            }
            // Object.handles includes queue-owned references. Ordinary handle
            // count above excludes those committed transfer holds.
            let queued = self.queued_references(id);
            usage.queued_references += queued;
            match owners {
                0 => {
                    assert!(queued > 0);
                    usage.queued_only_bytes += bytes;
                }
                1 => usage.private_bytes += bytes,
                _ => usage.shared_bytes += bytes,
            }
        }
        usage
    }
}
