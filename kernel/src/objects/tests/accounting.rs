use super::channels::{pair, receive, sender};
use super::*;
use crate::objects::channels::{Attachment, ENDPOINT_RIGHTS, Mode};
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
    let private = registry.backing_usage();
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
    assert_eq!(registry.backing_usage().queued_references, 1);
    let (_, handles, _) = receive(&mut registry, b, remote);
    let foreign = handles[0];
    let shared = registry.backing_usage();
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
    let queued = registry.backing_usage();
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
    let orphan = registry.backing_usage();
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
    let empty = registry.backing_usage();
    assert_eq!(
        empty.private_bytes + empty.shared_bytes + empty.queued_only_bytes,
        0
    );
    assert_eq!(runtime.inventory().resident, 0);
    assert!(ram.pages.is_empty());
}
