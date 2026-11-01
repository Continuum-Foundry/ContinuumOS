//! Host production-codec consumer. The reference implementation lives in Python.
use continuum_rt::wire::*;
use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let mut fields = line.split_whitespace();
        let kind = fields.next().unwrap();
        let hex = fields.next().unwrap_or("");
        let bytes: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        match validate(kind, &bytes) {
            Err(e) => println!("{e} -"),
            Ok(()) => {
                // Decode/re-encode through actual production constructors.
                let canonical: Vec<u8> = match kind {
                    "header" => {
                        let h = Header::decode(&bytes, 1).unwrap();
                        [h.encode().as_slice(), &bytes[48..]].concat()
                    }
                    "map" => {
                        let w = read_words::<4>(&bytes).unwrap();
                        map(w[0], w[1], w[2], w[3]).unwrap().to_vec()
                    }
                    "attachment" => attachment(
                        Handle(u64::from_le_bytes(bytes[..8].try_into().unwrap())),
                        u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
                        u32::from_le_bytes(bytes[16..20].try_into().unwrap()),
                    )
                    .unwrap()
                    .to_vec(),
                    "timer" => {
                        let w = read_words::<3>(&bytes).unwrap();
                        timer(MonotonicNs(w[0]), w[1]).unwrap().to_vec()
                    }
                    "wait_item" => {
                        let w = read_words::<4>(&bytes).unwrap();
                        wait_item(Handle(w[0]), w[1], w[2], w[3]).unwrap().to_vec()
                    }
                    "discovery" | "send" | "wait" => {
                        words::<4, 32>(read_words::<4>(&bytes).unwrap()).to_vec()
                    }
                    "receive" => words::<6, 48>(read_words::<6>(&bytes).unwrap()).to_vec(),
                    "service_request" => words::<2, 16>(read_words::<2>(&bytes).unwrap()).to_vec(),
                    "metadata" => words::<8, 64>(read_words::<8>(&bytes).unwrap()).to_vec(),
                    "attempt" => words::<3, 24>(read_words::<3>(&bytes).unwrap()).to_vec(),
                    "context" => words::<5, 40>(read_words::<5>(&bytes).unwrap()).to_vec(),
                    "route" => words::<20, 160>(read_words::<20>(&bytes).unwrap()).to_vec(),
                    _ => bytes,
                };
                let hex: String = canonical.iter().map(|b| format!("{b:02x}")).collect();
                println!("0 {hex}");
            }
        }
    }
}
