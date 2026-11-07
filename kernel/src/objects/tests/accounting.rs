use super::channels::{pair, receive, sender};
use super::*;
use crate::objects::channels::{Attachment, ENDPOINT_RIGHTS, Mode};

// Object-centric oracle retained from the original implementation. It searches
// for exact IDs independently of the new reference-centric accumulation.
fn reference(c: &Registry) -> crate::objects::accounting::BackingUsage {
    let mut expected = crate::objects::accounting::BackingUsage {
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
    for (slot, object) in c.objects.iter().enumerate() {
        expected.timers += usize::from(matches!(object.body, Some(Body::Timer(_))));
        expected.events += usize::from(matches!(object.body, Some(Body::Event(_))));
        let Some(Body::Memory { bytes, .. }) = object.body else {
            continue;
        };
        let id = Id {
            slot,
            generation: object.generation,
        };
        let mut owners = 0;
        for (owner, table) in c.tables.iter().enumerate() {
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
            expected.memory_handles += handles;
            expected.memory_maps += maps;
            if handles + maps != 0 {
                owners += 1;
                expected.owner_attribution[owner] += bytes;
            }
        }
        let queued = c.queued_references(id);
        expected.queued_references += queued;
        match owners {
            0 => {
                assert!(queued > 0);
                expected.queued_only_bytes += bytes;
            }
            1 => expected.private_bytes += bytes,
            _ => expected.shared_bytes += bytes,
        }
    }
    expected
}
fn usage(c: &Registry) -> crate::objects::accounting::BackingUsage {
    let actual = c.backing_usage();
    assert_eq!(actual, reference(c));
    actual
}
#[test]
fn physical_backing_is_partitioned_once_across_duplicate_transfer_and_orphan_lifetimes() {
    let (mut runtime, mut ram, mut registry) = setup();
    let a = runtime.create().unwrap();
    let b = runtime.create().unwrap();
    let (process, _) = registry.admit_sender(a, sender(0)).unwrap();
    registry.admit_sender(b, sender(1)).unwrap();
    let [tx, rx] = pair(&mut registry, a, process);
    let remote = registry.grant_endpoint(a, rx, b, ENDPOINT_RIGHTS).unwrap();
    registry.close(&mut runtime, &mut ram, a, rx).unwrap();
    let gift = registry
        .create_transferable(&mut runtime, &mut ram, a, process, PAGE, READ)
        .unwrap();
    let alias = registry
        .duplicate(a, gift, BASIC | READ | TRANSFER)
        .unwrap();
    let private = usage(&registry);
    assert_eq!(runtime.owner_backing(a).unwrap(), (PAGE, PAGE));
    assert_eq!(runtime.owner_backing(b).unwrap(), (0, 0));
    assert_eq!(
        (
            private.private_bytes,
            private.shared_bytes,
            private.queued_only_bytes
        ),
        (PAGE, 0, 0)
    );
    assert_eq!(private.owner_attribution.iter().sum::<u64>(), PAGE);
    assert_eq!(private.memory_handles, 2);
    registry
        .channel_send(
            a,
            tx,
            b"share",
            &[Attachment {
                handle: gift,
                rights: BASIC | READ,
                mode: Mode::Duplicate,
            }],
        )
        .unwrap();
    assert_eq!(usage(&registry).queued_references, 1);
    let (_, handles, _) = receive(&mut registry, b, remote);
    let foreign = handles[0];
    let shared = usage(&registry);
    assert_eq!(
        (
            shared.private_bytes,
            shared.shared_bytes,
            shared.queued_only_bytes
        ),
        (0, PAGE, 0)
    );
    assert_eq!(shared.owner_attribution.iter().sum::<u64>(), 2 * PAGE);
    let moved = registry
        .create_transferable(&mut runtime, &mut ram, a, process, PAGE, READ)
        .unwrap();
    registry
        .channel_send(
            a,
            tx,
            b"held",
            &[Attachment {
                handle: moved,
                rights: BASIC | READ,
                mode: Mode::Move,
            }],
        )
        .unwrap();
    let queued = usage(&registry);
    assert_eq!(
        (
            queued.private_bytes,
            queued.shared_bytes,
            queued.queued_only_bytes
        ),
        (0, PAGE, PAGE)
    );
    assert_eq!(queued.queued_references, 1);
    assert_eq!(registry.inventory().backing_bytes, 2 * PAGE);
    registry.close(&mut runtime, &mut ram, a, gift).unwrap();
    registry.close(&mut runtime, &mut ram, a, alias).unwrap();
    registry.remove(&mut runtime, &mut ram, a).unwrap();
    runtime.destroy(&mut ram, a).unwrap();
    let orphan = usage(&registry);
    assert_eq!(runtime.owner_backing(b).unwrap(), (0, 0));
    assert_eq!(registry.referenced_bytes(b).unwrap(), PAGE);
    assert_eq!(
        (
            orphan.private_bytes,
            orphan.shared_bytes,
            orphan.queued_only_bytes
        ),
        (PAGE, 0, PAGE)
    );
    assert_eq!(runtime.inventory().resident, 2 * PAGE);
    registry.close(&mut runtime, &mut ram, b, foreign).unwrap();
    registry.remove(&mut runtime, &mut ram, b).unwrap();
    runtime.destroy(&mut ram, b).unwrap();
    let empty = usage(&registry);
    assert_eq!(
        empty.private_bytes + empty.shared_bytes + empty.queued_only_bytes,
        0
    );
    assert_eq!(runtime.inventory().resident, 0);
    assert!(ram.pages.is_empty());
}

#[test]
fn backing_accounting_matches_object_oracle_through_queue_wrap_maps_and_slot_reuse() {
    let (mut r, mut ram, mut c) = setup();
    let a = r.create().unwrap();
    let b = r.create().unwrap();
    let (p, _) = c.admit_sender(a, sender(0)).unwrap();
    let (q, _) = c.admit_sender(b, sender(1)).unwrap();
    let [tx, rx] = pair(&mut c, a, p);
    let remote = c.grant_endpoint(a, rx, b, ENDPOINT_RIGHTS).unwrap();
    c.close(&mut r, &mut ram, a, rx).unwrap();
    // Retain multiple sizes/aliases; one object is counted once per owner even
    // when both handles and mappings refer to it. Timers/events are separate.
    let mut gifts = vec![];
    for n in 1..=8 {
        let h = c
            .create_transferable(&mut r, &mut ram, a, p, n * PAGE, READ)
            .unwrap();
        c.duplicate(a, h, BASIC | MAP | READ).unwrap();
        c.map(
            &mut r,
            &mut ram,
            a,
            p,
            h,
            0x400000 + n * 0x10000,
            0,
            PAGE,
            Access::Read,
        )
        .unwrap();
        gifts.push(h);
        usage(&c);
    }
    c.event_create(a, p, false).unwrap();
    c.timer_create(a, p).unwrap();
    for cycle in 0..24 {
        let h = gifts[cycle % gifts.len()];
        c.channel_send(
            a,
            tx,
            b"alias",
            &[Attachment {
                handle: h,
                rights: BASIC | MAP | READ,
                mode: Mode::Duplicate,
            }],
        )
        .unwrap();
        usage(&c);
        let (_, handles, _) = receive(&mut c, b, remote);
        c.map(
            &mut r,
            &mut ram,
            b,
            q,
            handles[0],
            0x600000,
            0,
            PAGE,
            Access::Read,
        )
        .unwrap();
        c.close(&mut r, &mut ram, b, handles[0]).unwrap();
        let shared = usage(&c);
        assert!(shared.shared_bytes > 0);
        c.unmap(&mut r, &mut ram, b, q, 0x600000).unwrap();
        usage(&c);
        // Move-only objects survive solely in a wrapped queue and are reclaimed
        // after reception/close. The freed slots are reused with new generations.
        let moved = c
            .create_transferable(&mut r, &mut ram, a, p, PAGE, READ)
            .unwrap();
        c.channel_send(
            a,
            tx,
            b"move",
            &[Attachment {
                handle: moved,
                rights: BASIC | READ,
                mode: Mode::Move,
            }],
        )
        .unwrap();
        assert_eq!(usage(&c).queued_only_bytes, PAGE);
        let (_, handles, _) = receive(&mut c, b, remote);
        c.close(&mut r, &mut ram, b, handles[0]).unwrap();
        usage(&c);
    }
    let before = c.inventory();
    let start = std::time::Instant::now();
    for _ in 0..1024 {
        std::hint::black_box(c.backing_usage());
    }
    let fast = start.elapsed();
    let start = std::time::Instant::now();
    for _ in 0..1024 {
        std::hint::black_box(reference(&c));
    }
    std::println!(
        "backing-accounting host diagnostic: single-pass={fast:?} object-scan={:?}",
        start.elapsed()
    );
    assert_eq!(c.inventory(), before);
    c.remove(&mut r, &mut ram, a).unwrap();
    r.destroy(&mut ram, a).unwrap();
    usage(&c);
    c.remove(&mut r, &mut ram, b).unwrap();
    r.destroy(&mut ram, b).unwrap();
    assert_eq!(usage(&c), Registry::EMPTY.backing_usage());
    assert!(ram.pages.is_empty());
}
