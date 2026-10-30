use super::channels::{pair, receive, sender};
use super::*;
use crate::objects::channels::{Attachment, Mode};
#[test]
fn constrained_receive_and_pair_copy_failures_leave_authority_and_credits_unchanged() {
    let (mut r, mut b, mut c) = setup();
    let s = r.create().unwrap();
    let (p, _) = c.admit_sender(s, sender(0)).unwrap();
    c.constrain(s).unwrap();
    let before = c.inventory();
    assert_eq!(
        c.channel_pair(s, p, |_| Err(Error::InvalidArgument)),
        Err(Error::InvalidArgument)
    );
    assert_eq!(c.inventory(), before);
    let [tx, rx] = pair(&mut c, s, p);
    let gift = c.event_create(s, p, false).unwrap();
    c.channel_send(
        s,
        tx,
        b"held",
        &[Attachment {
            handle: gift,
            rights: BASIC,
            mode: Mode::Move,
        }],
    )
    .unwrap();
    let mut handles = vec![];
    while let Ok(h) = c.duplicate(s, p, BASIC) {
        handles.push(h);
    }
    let inventory = c.inventory();
    let usage = c.channel_usage();
    assert_eq!(
        c.channel_receive(s, rx, 4096, 8, |_, _, _| panic!("copy must not run")),
        Err(Error::QuotaExceeded)
    );
    assert_eq!(c.inventory(), inventory);
    assert_eq!(c.channel_usage(), usage);
    c.close(&mut r, &mut b, s, handles.pop().unwrap()).unwrap();
    let (_, held, _) = receive(&mut c, s, rx);
    assert_eq!(held.len(), 1);
    c.remove(&mut r, &mut b, s).unwrap();
    r.destroy(&mut b, s).unwrap();
    assert_eq!(c.inventory().objects, 0);
    assert!(b.pages.is_empty());
}
#[test]
fn credits_survive_close_send_failure_and_creator_death_until_references_settle() {
    let (mut r, mut b, mut c) = setup();
    let s = r.create().unwrap();
    let (p, _) = c.admit_sender(s, sender(0)).unwrap();
    c.constrain(s).unwrap();
    let other = r.create().unwrap();
    c.admit_sender(other, sender(1)).unwrap();
    c.constrain(other).unwrap();
    let [tx, rx] = pair(&mut c, s, p);
    let rx = c
        .grant_endpoint(s, rx, other, crate::objects::channels::ENDPOINT_RIGHTS)
        .unwrap();
    let h = c
        .create_transferable(&mut r, &mut b, s, p, PAGE, READ)
        .unwrap();
    let attachment = [Attachment {
        handle: h,
        rights: BASIC | READ,
        mode: Mode::Move,
    }];
    let before = c.inventory();
    assert_eq!(
        c.channel_send_checked(s, tx, b"x", &attachment, || Err(Error::WouldBlock)),
        Err(Error::WouldBlock)
    );
    assert_eq!(c.inventory(), before);
    c.info(s, h).unwrap();
    c.channel_send(s, tx, &[1; 4096], &attachment).unwrap();
    c.channel_send(s, tx, &[2; 4096], &[]).unwrap();
    assert_eq!(c.channel_send(s, tx, b"x", &[]), Err(Error::QuotaExceeded));
    c.channel_send(s, tx, b"", &[]).unwrap();
    c.channel_send(s, tx, b"", &[]).unwrap();
    assert_eq!(c.channel_send(s, tx, b"", &[]), Err(Error::QuotaExceeded));
    c.remove(&mut r, &mut b, s).unwrap();
    r.destroy(&mut b, s).unwrap();
    assert_eq!(c.inventory().backing_bytes, PAGE);
    assert_eq!(receive(&mut c, other, rx).1.len(), 1);
    c.remove(&mut r, &mut b, other).unwrap();
    r.destroy(&mut b, other).unwrap();
    assert_eq!(c.inventory().objects, 0);
    assert_eq!(c.channel_usage().messages, 0);
    assert!(b.pages.is_empty());
}
#[test]
fn lifetime_object_timer_pair_credits_do_not_refund_and_control_owner_retains_room() {
    let (mut r, mut b, mut c) = setup();
    let s = r.create().unwrap();
    let (p, _) = c.admit(s).unwrap();
    c.constrain(s).unwrap();
    for _ in 0..4 {
        let h = c.timer_create(s, p).unwrap();
        c.close(&mut r, &mut b, s, h).unwrap();
    }
    assert_eq!(c.timer_create(s, p), Err(Error::QuotaExceeded));
    let pair = pair(&mut c, s, p);
    for h in pair {
        c.close(&mut r, &mut b, s, h).unwrap();
    }
    assert_eq!(
        c.channel_pair(s, p, |_| panic!("no copy")),
        Err(Error::QuotaExceeded)
    );
    for _ in 0..4 {
        let h = c.event_create(s, p, false).unwrap();
        c.close(&mut r, &mut b, s, h).unwrap();
    }
    assert_eq!(c.event_create(s, p, false), Err(Error::QuotaExceeded));
    let control = r.create().unwrap();
    let (q, _) = c.admit(control).unwrap();
    c.constrain(control).unwrap();
    c.event_create(control, q, true).unwrap();
    c.info(s, p).unwrap();
}
