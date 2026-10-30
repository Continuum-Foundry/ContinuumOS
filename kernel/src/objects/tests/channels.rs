use super::*;
use crate::objects::channels::*;
pub(super) fn sender(process: u64) -> Sender {
    Sender {
        process,
        generation: 1,
        principal: if process == 1 { 202 } else { 101 },
        session: if process == 1 { 2002 } else { 1001 },
        service: false,
    }
}
pub(super) fn pair(c: &mut Registry, s: SpaceId, p: u64) -> [u64; 2] {
    let mut h = [0; 2];
    c.channel_pair(s, p, |v| {
        h = v;
        Ok(())
    })
    .unwrap();
    h
}
pub(super) fn receive(c: &mut Registry, s: SpaceId, h: u64) -> (Vec<u8>, Vec<u64>, Sender) {
    let mut out = (vec![], vec![], sender(0));
    c.channel_receive(s, h, INLINE, ATTACHMENTS, |b, h, s| {
        out = (b.to_vec(), h.to_vec(), s);
        Ok(())
    })
    .unwrap();
    out
}
#[test]
fn final_admission_failure_preserves_bytes_authority_and_nonce() {
    let (mut r, mut b, mut c) = setup();
    let s = r.create().unwrap();
    let (p, _) = c.admit_sender(s, sender(0)).unwrap();
    let [tx, rx] = pair(&mut c, s, p);
    let gift = c
        .create_transferable(&mut r, &mut b, s, p, PAGE, READ)
        .unwrap();
    let before = c.inventory();
    let usage = c.channel_usage();
    let nonce = c.next_nonce;
    let attachments = [Attachment {
        handle: gift,
        rights: BASIC | READ,
        mode: Mode::Move,
    }];
    assert_eq!(
        c.channel_send_checked(s, tx, b"atomic", &attachments, || Err(Error::WouldBlock)),
        Err(Error::WouldBlock)
    );
    assert_eq!(c.inventory(), before);
    assert_eq!(c.channel_usage(), usage);
    assert_eq!(c.next_nonce, nonce);
    assert!(c.info(s, gift).is_ok());
    c.channel_send_checked(s, tx, b"atomic", &attachments, || Ok(()))
        .unwrap();
    assert_eq!(c.info(s, gift), Err(Error::BadHandle));
    let (bytes, handles, _) = receive(&mut c, s, rx);
    assert_eq!(bytes, b"atomic");
    assert_eq!(handles.len(), 1);
}
fn attach(handle: u64, rights: u64, mode: Mode) -> Attachment {
    Attachment {
        handle,
        rights,
        mode,
    }
}
#[test]
fn atomic_send_receive_copies_and_attenuation() {
    let (mut r, mut b, mut c) = setup();
    let a = r.create().unwrap();
    let z = r.create().unwrap();
    let (p, _) = c.admit_sender(a, sender(0)).unwrap();
    c.admit_sender(z, sender(1)).unwrap();
    let [tx, rx] = pair(&mut c, a, p);
    let remote = c.grant_endpoint(a, rx, z, ENDPOINT_RIGHTS).unwrap();
    c.close(&mut r, &mut b, a, rx).unwrap();
    let [gift, other] = pair(&mut c, a, p);
    let inv = c.inventory();
    let used = c.channel_usage();
    let bad = [
        attach(gift, BASIC, Mode::Move),
        attach(u64::MAX, BASIC, Mode::Move),
    ];
    assert_eq!(c.channel_send(a, tx, b"no", &bad), Err(Error::BadHandle));
    assert_eq!(c.inventory(), inv);
    assert_eq!(c.channel_usage(), used);
    assert_eq!(
        c.channel_send(a, tx, b"no", &[attach(gift, u64::MAX, Mode::Move)]),
        Err(Error::AccessDenied)
    );
    assert_eq!(
        c.channel_send(
            a,
            tx,
            b"no",
            &[
                attach(gift, BASIC, Mode::Move),
                attach(gift, BASIC, Mode::Duplicate)
            ]
        ),
        Err(Error::InvalidArgument)
    );
    c.channel_send(
        a,
        tx,
        b"owner=999;role=admin",
        &[attach(gift, BASIC | SEND, Mode::Move)],
    )
    .unwrap();
    assert_eq!(c.info(a, gift), Err(Error::BadHandle));
    assert_eq!(c.channel_size(z, remote), Ok((20, 1)));
    let pending = c.inventory();
    let quota = c.channel_usage();
    let nonce = c.next_nonce;
    assert_eq!(
        c.channel_receive(z, remote, 0, 1, |_, _, _| panic!()),
        Err(Error::BufferTooSmall)
    );
    assert_eq!(
        c.channel_receive(z, remote, INLINE, 0, |_, _, _| panic!()),
        Err(Error::BufferTooSmall)
    );
    assert_eq!(
        c.channel_receive(z, remote, INLINE, 1, |_, _, _| Err(Error::InvalidArgument)),
        Err(Error::InvalidArgument)
    );
    assert_eq!(c.inventory(), pending);
    assert_eq!(c.channel_usage(), quota);
    assert_eq!(c.next_nonce, nonce);
    let (bytes, handles, trusted) = receive(&mut c, z, remote);
    assert_eq!(bytes, b"owner=999;role=admin");
    assert_eq!(trusted, sender(0));
    assert_eq!(
        c.info(z, handles[0]),
        Ok((Kind::Channel as u64) << 32 | BASIC | SEND)
    );
    assert_eq!(c.channel_send(z, handles[0], b"delegated", &[]), Ok(()));
    assert_eq!(receive(&mut c, a, other).2, sender(1));
    assert_eq!(
        c.channel_send(
            z,
            handles[0],
            b"no",
            &[attach(handles[0], BASIC, Mode::Move)]
        ),
        Err(Error::AccessDenied)
    );
    c.remove(&mut r, &mut b, a).unwrap();
    c.remove(&mut r, &mut b, z).unwrap();
    r.destroy(&mut b, a).unwrap();
    r.destroy(&mut b, z).unwrap();
    assert_eq!(c.inventory().objects, 0);
    assert_eq!(
        c.channel_usage(),
        Usage {
            pairs: 0,
            messages: 0,
            bytes: 0,
            handles: 0
        }
    );
}
#[test]
fn queue_count_byte_handle_limits_are_independent_and_preserve_sources() {
    let (mut r, mut b, mut c) = setup();
    let s = r.create().unwrap();
    let (p, _) = c.admit_sender(s, sender(0)).unwrap();
    let [tx, rx] = pair(&mut c, s, p);
    for _ in 0..MESSAGES {
        c.channel_send(s, tx, &[], &[]).unwrap();
    }
    assert_eq!(c.channel_send(s, tx, &[], &[]), Err(Error::WouldBlock));
    for _ in 0..MESSAGES {
        receive(&mut c, s, rx);
    }
    for _ in 0..4 {
        c.channel_send(s, tx, &[7; INLINE], &[]).unwrap();
    }
    assert_eq!(c.channel_send(s, tx, &[7], &[]), Err(Error::WouldBlock));
    for _ in 0..4 {
        receive(&mut c, s, rx);
    }
    let gifts: Vec<_> = (0..8)
        .map(|_| c.duplicate(s, rx, ENDPOINT_RIGHTS).unwrap())
        .collect();
    let list: Vec<_> = gifts
        .iter()
        .map(|h| attach(*h, BASIC, Mode::Duplicate))
        .collect();
    c.channel_send(s, tx, &[], &list).unwrap();
    c.channel_send(s, tx, &[], &list).unwrap();
    let inv = c.inventory();
    let u = c.channel_usage();
    assert_eq!(
        c.channel_send(s, tx, &[], &[attach(gifts[0], BASIC, Mode::Move)]),
        Err(Error::WouldBlock)
    );
    assert_eq!(c.inventory(), inv);
    assert_eq!(c.channel_usage(), u);
    assert!(c.info(s, gifts[0]).is_ok());
    assert_eq!(
        c.channel_send(s, tx, &[0; INLINE + 1], &[]),
        Err(Error::InvalidArgument)
    );
    assert_eq!(
        c.channel_send(s, tx, &[], &[attach(rx, BASIC, Mode::Duplicate); 9]),
        Err(Error::InvalidArgument)
    );
    c.remove(&mut r, &mut b, s).unwrap();
    r.destroy(&mut b, s).unwrap();
    assert_eq!(c.inventory().objects, 0);
}
#[test]
fn receive_table_and_nonce_exhaustion_retain_entire_message() {
    let (mut r, mut b, mut c) = setup();
    let s = r.create().unwrap();
    let (p, _) = c.admit_sender(s, sender(0)).unwrap();
    let [tx, rx] = pair(&mut c, s, p);
    c.channel_send(
        s,
        tx,
        b"x",
        &[
            attach(tx, BASIC, Mode::Duplicate),
            attach(rx, BASIC, Mode::Duplicate),
        ],
    )
    .unwrap();
    let mut filled = vec![];
    while let Ok(h) = c.duplicate(s, tx, BASIC) {
        filled.push(h);
    }
    let inv = c.inventory();
    assert_eq!(
        c.channel_receive(s, rx, INLINE, 8, |_, _, _| panic!()),
        Err(Error::QuotaExceeded)
    );
    c.close(&mut r, &mut b, s, filled.pop().unwrap()).unwrap();
    assert_eq!(
        c.channel_receive(s, rx, INLINE, 8, |_, _, _| panic!()),
        Err(Error::QuotaExceeded)
    );
    c.close(&mut r, &mut b, s, filled.pop().unwrap()).unwrap();
    c.next_nonce = u32::MAX;
    assert_eq!(
        c.channel_receive(s, rx, INLINE, 8, |_, _, _| panic!()),
        Err(Error::QuotaExceeded)
    );
    assert_eq!(c.inventory().handles, inv.handles - 2);
    assert_eq!(c.channel_usage().messages, 1);
    c.next_nonce = u32::MAX - 1;
    let (_, h, _) = receive(&mut c, s, rx);
    assert_eq!(h[1] >> 32, u32::MAX as u64);
    assert_eq!(c.next_nonce, 0);
    // Empty attachment lists remain receivable after handle nonce exhaustion.
    c.channel_send(s, tx, b"ok", &[]).unwrap();
    assert_eq!(receive(&mut c, s, rx).0, b"ok");
    c.remove(&mut r, &mut b, s).unwrap();
    r.destroy(&mut b, s).unwrap();
    assert_eq!(c.inventory().objects, 0);
}
#[test]
fn pair_copy_and_admission_failure_install_nothing() {
    let (mut r, mut b, mut c) = setup();
    let s = r.create().unwrap();
    let (p, _) = c.admit_sender(s, sender(0)).unwrap();
    let inv = c.inventory();
    let nonce = c.next_nonce;
    assert_eq!(
        c.channel_pair(s, p, |_| Err(Error::InvalidArgument)),
        Err(Error::InvalidArgument)
    );
    assert_eq!(c.inventory(), inv);
    assert_eq!(c.next_nonce, nonce);
    assert_eq!(c.channel_usage().pairs, 0);
    for _ in 0..PAIRS {
        pair(&mut c, s, p);
    }
    let full = c.inventory();
    assert_eq!(
        c.channel_pair(s, p, |_| panic!()),
        Err(Error::QuotaExceeded)
    );
    assert_eq!(c.inventory(), full);
    c.remove(&mut r, &mut b, s).unwrap();
    r.destroy(&mut b, s).unwrap();
    assert_eq!(c.inventory().objects, 0);
}
#[test]
fn sender_death_preserves_queued_metadata_and_receiver_death_discards_queued_refs() {
    let (mut r, mut b, mut c) = setup();
    let a = r.create().unwrap();
    let z = r.create().unwrap();
    let (p, _) = c.admit_sender(a, sender(0)).unwrap();
    c.admit_sender(z, sender(1)).unwrap();
    let [tx, rx] = pair(&mut c, a, p);
    let remote = c.grant_endpoint(a, rx, z, ENDPOINT_RIGHTS).unwrap();
    c.close(&mut r, &mut b, a, rx).unwrap();
    c.channel_send(a, tx, b"accepted", &[attach(tx, BASIC, Mode::Move)])
        .unwrap();
    c.remove(&mut r, &mut b, a).unwrap();
    r.destroy(&mut b, a).unwrap();
    assert_eq!(c.channel_usage().messages, 1);
    assert_eq!(receive(&mut c, z, remote).2, sender(0));
    assert_eq!(c.channel_size(z, remote), Err(Error::WouldBlock)); // moved tx remains installed
    c.remove(&mut r, &mut b, z).unwrap();
    r.destroy(&mut b, z).unwrap();
    assert_eq!(c.inventory().objects, 0);
}
#[test]
fn abandoned_endpoint_cycles_are_collected_and_empty_closed_peer_is_typed() {
    let (mut r, mut b, mut c) = setup();
    let s = r.create().unwrap();
    let (p, _) = c.admit_sender(s, sender(0)).unwrap();
    let [tx, rx] = pair(&mut c, s, p);
    c.channel_send(s, tx, b"cycle", &[attach(rx, ENDPOINT_RIGHTS, Mode::Move)])
        .unwrap();
    assert_eq!(c.channel_usage().messages, 1);
    c.close(&mut r, &mut b, s, tx).unwrap();
    assert_eq!(c.channel_usage().messages, 0);
    assert_eq!(c.channel_usage().pairs, 0);
    assert_eq!(c.inventory().objects, 2);
    let [tx, rx] = pair(&mut c, s, p);
    c.close(&mut r, &mut b, s, rx).unwrap();
    assert_eq!(c.channel_send(s, tx, b"x", &[]), Err(Error::PeerClosed));
    assert_eq!(c.channel_size(s, tx), Err(Error::PeerClosed));
    c.remove(&mut r, &mut b, s).unwrap();
    r.destroy(&mut b, s).unwrap();
    assert_eq!(c.inventory().objects, 0);
}
#[test]
fn envelope_independent_literal_and_malformed_inputs() {
    use crate::ipc_wire::{Envelope, Error as W};
    let mut bytes = [0; 64];
    let e = Envelope {
        protocol: 0x12345678,
        major: 1,
        minor: 2,
        operation: 0x90abcdef,
        required_features: 1,
        request_id: 0x0807060504030201,
        route_epoch: 3,
        policy_epoch: 4,
    };
    let n = e.encode(b"abc", 1, &mut bytes).unwrap();
    let literal = [
        0x78, 0x56, 0x34, 0x12, 1, 0, 2, 0, 0xef, 0xcd, 0xab, 0x90, 1, 0, 0, 0, 3, 0, 0, 0, 1, 0,
        0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 3, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 97, 98, 99,
    ];
    assert_eq!(&bytes[..n], literal);
    assert_eq!(
        Envelope::decode(&literal, 1, e.protocol, 1, 2, 1),
        Ok((e, &b"abc"[..]))
    );
    for size in 0..51 {
        assert!(Envelope::decode(&literal[..size], 1, e.protocol, 1, 2, 1).is_err());
    }
    for (offset, value, expected) in [
        (22, 1, W::Reserved),
        (16, 255, W::Length),
        (20, 9, W::Handles),
        (12, 2, W::Features),
        (4, 2, W::Protocol),
        (6, 3, W::Protocol),
        (0, 0, W::Protocol),
    ] {
        let mut bad = literal;
        bad[offset] = value;
        assert_eq!(
            Envelope::decode(&bad, 1, e.protocol, 1, 2, 1),
            Err(expected)
        );
    }
    assert_eq!(
        Envelope::decode(&bytes, 1, e.protocol, 1, 2, 1),
        Err(W::Length)
    );
    assert_eq!(
        Envelope::decode(&literal, 0, e.protocol, 1, 2, 1),
        Err(W::Handles)
    );
}

#[test]
fn receiver_death_drops_queued_memory_and_cross_pair_cycles() {
    let (mut r, mut b, mut c) = setup();
    let a = r.create().unwrap();
    let z = r.create().unwrap();
    let (p, _) = c.admit_sender(a, sender(0)).unwrap();
    c.admit_sender(z, sender(1)).unwrap();
    let [tx, rx] = pair(&mut c, a, p);
    let _remote = c.grant_endpoint(a, rx, z, ENDPOINT_RIGHTS).unwrap();
    c.close(&mut r, &mut b, a, rx).unwrap();
    let h = c
        .create_transferable(&mut r, &mut b, a, p, 4096, READ)
        .unwrap();
    c.channel_send(
        a,
        tx,
        b"queued",
        &[attach(h, BASIC | MAP | READ, Mode::Move)],
    )
    .unwrap();
    assert_eq!(c.info(a, h), Err(Error::BadHandle));
    assert_eq!(c.inventory().backing_bytes, 4096);
    c.remove(&mut r, &mut b, z).unwrap();
    r.destroy(&mut b, z).unwrap();
    assert_eq!(c.channel_usage().handles, 0);
    assert_eq!(c.inventory().backing_bytes, 0);
    assert_eq!(
        c.channel_send(a, tx, b"lost peer", &[]),
        Err(Error::PeerClosed)
    );
    c.close(&mut r, &mut b, a, tx).unwrap();
    let [a0, a1] = pair(&mut c, a, p);
    let [b0, b1] = pair(&mut c, a, p);
    c.channel_send(a, a0, b"a", &[attach(b1, ENDPOINT_RIGHTS, Mode::Move)])
        .unwrap();
    c.channel_send(a, b0, b"b", &[attach(a1, ENDPOINT_RIGHTS, Mode::Move)])
        .unwrap();
    assert_eq!(c.channel_usage().messages, 2);
    c.close(&mut r, &mut b, a, a0).unwrap();
    c.close(&mut r, &mut b, a, b0).unwrap();
    assert_eq!(c.channel_usage().pairs, 0);
    assert_eq!(c.inventory().objects, 2);
    c.remove(&mut r, &mut b, a).unwrap();
    r.destroy(&mut b, a).unwrap();
    assert!(b.pages.is_empty());
    assert_eq!(c.inventory().objects, 0);
}
