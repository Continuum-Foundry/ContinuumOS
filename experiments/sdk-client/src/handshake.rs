//! Prior compatible consumer. This file is unchanged for both service versions.
use super::*;
use sdk::wire::{Handle, Header, PolicyEpoch, RouteEpoch};
fn packet(n: u64) -> [u8; 64] {
    let mut h = Header {
        protocol: 0x53444b,
        major: 1,
        minor: 0,
        operation: 1,
        required_features: 0,
        payload_bytes: 16,
        handles: 0,
        request: n,
        route: RouteEpoch(0),
        policy: PolicyEpoch(0),
    };
    if n == 2 {
        h.minor = 99;
        h.operation = 2;
    }
    if n == 3 {
        h.major = 2;
    }
    if n == 4 {
        h.required_features = 2;
    }
    if n == 6 || n == 8 {
        h.operation = 3;
        h.handles = 1;
    }
    if n == 7 {
        h.required_features = 1;
    }
    if n == 8 {
        h.major = 2;
    }
    let mut b = [0; 64];
    b[..48].copy_from_slice(&h.encode());
    if n == 5 {
        b[22] = 1;
    }
    b[48..56].copy_from_slice(&n.to_le_bytes());
    if n == 9 {
        b[56] = 1;
    }
    if n == 10 {
        b[48..56].copy_from_slice(&(n + 1).to_le_bytes());
    }
    b
}
pub fn client() {
    // Negotiate each independently built version once. Later incarnations
    // retain the unchanged client bytes for the route/policy failure schedule.
    if config(0) != 0 {
        ok(HANDLE_CLOSE, config(4), 0, 0, 0);
        return;
    }
    for actor in [4usize, 5] {
        let path = config(if actor == 4 { 3 } else { 4 });
        for n in 1..=10 {
            let b = packet(n);
            let mut attachment = [0; 24];
            let mut source = 0;
            if n == 6 || n == 8 {
                // Immutable zero-filled object moved atomically, not a native struct.
                source = ok(MEMORY_CREATE, word(3), 4096, 32, 0);
                attachment = sdk::wire::attachment(Handle(source), 1 | 4 | 16 | 32, 0).unwrap();
            }
            let d = sdk::wire::words::<4, 32>([
                b.as_ptr() as u64,
                64,
                if n == 6 || n == 8 {
                    attachment.as_ptr() as u64
                } else {
                    0
                },
                u64::from(n == 6 || n == 8),
            ]);
            loop {
                let (e, _) = invoke(CHANNEL_SEND, path, d.as_ptr() as u64, 32, 0);
                if e == WOULDBLOCK {
                    yield_now();
                    continue;
                }
                require(e == 0);
                break;
            }
            if n == 6 || n == 8 {
                require(invoke(OBJECT_INFO, source, 0, 0, 0).0 == BADHANDLE);
            }
            wait_handle(path, 2);
            let mut reply = [0; 64];
            let mut meta = [0; 64];
            let d = sdk::wire::words::<6, 48>([
                reply.as_mut_ptr() as u64,
                64,
                0,
                0,
                meta.as_mut_ptr() as u64,
                64,
            ]);
            ok(CHANNEL_RECEIVE, path, d.as_ptr() as u64, 48, 0);
            let m = sdk::wire::read_words::<8>(&meta).unwrap();
            let who = identity(actor);
            require(
                m[0] == 64
                    && m[1] == 0
                    && m[2] == who.process
                    && m[3] == who.generation
                    && m[4] == who.principal
                    && m[5] == who.session
                    && m[6] == 1
                    && m[7] == 0,
            );
            let h = Header::decode(&reply, 1).unwrap();
            require(h.request == n);
            let status = u64::from_le_bytes(reply[48..56].try_into().unwrap());
            require(
                status
                    == match n {
                        3 | 4 | 8 => 14,
                        5 | 9 | 10 => 4,
                        7 if actor == 4 => 14,
                        _ => 0,
                    },
            );
            let minor = u64::from_le_bytes(reply[56..].try_into().unwrap());
            let expected = u64::from(actor == 5);
            require(minor & 0xffffffff == expected && minor >> 32 == expected);
            trace(format_args!(
                "sdk-hello-{}-{}-{}-{}",
                config(0),
                actor,
                n,
                status
            ));
        }
        if actor == 5 {
            ok(HANDLE_CLOSE, path, 0, 0, 0);
        }
    }
}
