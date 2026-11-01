//! Independent harness over the production SDK API. Print decoded VALUES as
//! well as bytes: matching encoder/decoder field swaps must not pass round trips.
use continuum_rt::wire::*;
use std::io::{self, BufRead};
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn unhex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn csv(v: &[u64]) -> String {
    v.iter().map(u64::to_string).collect::<Vec<_>>().join(",")
}
fn decoded(kind: &str, b: &[u8]) -> Vec<u64> {
    match kind {
        "startup" => {
            let s = Startup::decode(b).unwrap();
            vec![s.bootstrap.0, s.clock_resolution_ns]
        }
        "header" => {
            let h = Header::decode(b, 1).unwrap();
            vec![
                u64::from(h.protocol),
                u64::from(h.major),
                u64::from(h.minor),
                u64::from(h.operation),
                u64::from(h.required_features),
                u64::from(h.payload_bytes),
                u64::from(h.handles),
                h.request,
                h.route.0,
                h.policy.0,
            ]
        }
        "attachment" => {
            let w = read_words::<3>(b).unwrap();
            vec![w[0], w[1], w[2] & 0xffffffff, w[2] >> 32]
        }
        "service_request" => read_words::<2>(b).unwrap().to_vec(),
        "timer" | "attempt" => read_words::<3>(b).unwrap().to_vec(),
        "discovery" | "map" | "send" | "wait" | "wait_item" => read_words::<4>(b).unwrap().to_vec(),
        "context" => read_words::<5>(b).unwrap().to_vec(),
        "receive" => read_words::<6>(b).unwrap().to_vec(),
        "metadata" => read_words::<8>(b).unwrap().to_vec(),
        "route" => read_words::<20>(b).unwrap().to_vec(),
        _ => unreachable!(),
    }
}
fn produce(kind: &str, w: &[u64], payload: &[u8]) -> Result<Vec<u8>, u64> {
    Ok(match kind {
        "header" => {
            let h = Header {
                protocol: w[0] as u32,
                major: w[1] as u16,
                minor: w[2] as u16,
                operation: w[3] as u32,
                required_features: w[4] as u32,
                payload_bytes: w[5] as u32,
                handles: w[6] as u16,
                request: w[7],
                route: RouteEpoch(w[8]),
                policy: PolicyEpoch(w[9]),
            };
            [h.encode().as_slice(), payload].concat()
        }
        "map" => map(w[0], w[1], w[2], w[3])?.to_vec(),
        "attachment" => attachment(Handle(w[0]), w[1], w[2] as u32)?.to_vec(),
        "timer" => timer(MonotonicNs(w[0]), w[1])?.to_vec(),
        "wait_item" => wait_item(Handle(w[0]), w[1], w[2], w[3])?.to_vec(),
        _ => match w.len() {
            2 => words::<2, 16>(w.try_into().unwrap()).to_vec(),
            3 => words::<3, 24>(w.try_into().unwrap()).to_vec(),
            4 => words::<4, 32>(w.try_into().unwrap()).to_vec(),
            5 => words::<5, 40>(w.try_into().unwrap()).to_vec(),
            6 => words::<6, 48>(w.try_into().unwrap()).to_vec(),
            8 => words::<8, 64>(w.try_into().unwrap()).to_vec(),
            20 => words::<20, 160>(w.try_into().unwrap()).to_vec(),
            _ => unreachable!(),
        },
    })
}
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let mut p = line.split_whitespace();
        let kind = p.next().unwrap();
        if let Some(kind) = kind.strip_prefix("encode-") {
            let w = p
                .next()
                .unwrap()
                .split(',')
                .map(|v| v.parse::<u64>().unwrap())
                .collect::<Vec<_>>();
            let payload = unhex(p.next().unwrap_or(""));
            match produce(kind, &w, &payload) {
                Ok(b) => println!("0 {} -", hex(&b)),
                Err(e) => println!("{e} - -"),
            };
        } else {
            let b = unhex(p.next().unwrap_or(""));
            match validate(kind, &b) {
                Ok(()) => println!("0 {} {}", hex(&b), csv(&decoded(kind, &b))),
                Err(e) => println!("{e} - -"),
            };
        }
    }
}
