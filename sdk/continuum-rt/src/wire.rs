//! Public encodings and value domains. Identity data is never a grant.
use crate::*;
macro_rules! domain { ($($name:ident),*) => { $(#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub struct $name(pub u64);)* }; }
domain!(
    Handle,
    ProcessGeneration,
    AuthoritySession,
    PolicyEpoch,
    RouteEpoch,
    MonotonicNs,
    WallClockNs
);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Identity {
    pub process: u64,
    pub generation: u64,
    pub principal: u64,
    pub session: u64,
    pub service: bool,
}
pub const NEVER: MonotonicNs = MonotonicNs(u64::MAX);
pub fn words<const N: usize, const B: usize>(values: [u64; N]) -> [u8; B] {
    assert_eq!(B, N * 8);
    let mut out = [0; B];
    for (i, value) in values.iter().enumerate() {
        out[i * 8..i * 8 + 8].copy_from_slice(&value.to_le_bytes());
    }
    out
}
pub fn read_words<const N: usize>(bytes: &[u8]) -> Result<[u64; N], u64> {
    if bytes.len() != N * 8 {
        return Err(INVALIDARGUMENT);
    }
    Ok(core::array::from_fn(|i| {
        u64::from_le_bytes(bytes[i * 8..i * 8 + 8].try_into().unwrap())
    }))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Startup {
    pub bootstrap: Handle,
    pub clock_resolution_ns: u64,
}
impl Startup {
    pub fn decode(b: &[u8]) -> Result<Self, u64> {
        if b.len() != 64
            || &b[..8] != b"CNTM3RT0"
            || b[8..12] != 0u32.to_le_bytes()
            || b[12..16] != 64u32.to_le_bytes()
            || b[16..24] != [0; 8]
            || b[32..36] != 4096u32.to_le_bytes()
            || b[36..40] != [0; 4]
            || b[48..] != [0; 16]
        {
            return Err(INCOMPATIBLEPROTOCOL);
        }
        Ok(Self {
            bootstrap: Handle(u64::from_le_bytes(b[24..32].try_into().unwrap())),
            clock_resolution_ns: u64::from_le_bytes(b[40..48].try_into().unwrap()),
        })
    }
}
/// 48-byte version header plus bounded payload. Unknown optional minor versions
/// are compatible; every required feature must be present at this receiver.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub protocol: u32,
    pub major: u16,
    pub minor: u16,
    pub operation: u32,
    pub required_features: u32,
    pub payload_bytes: u32,
    pub handles: u16,
    pub request: u64,
    pub route: RouteEpoch,
    pub policy: PolicyEpoch,
}
impl Header {
    pub fn encode(self) -> [u8; 48] {
        let mut b = [0; 48];
        b[..4].copy_from_slice(&self.protocol.to_le_bytes());
        b[4..6].copy_from_slice(&self.major.to_le_bytes());
        b[6..8].copy_from_slice(&self.minor.to_le_bytes());
        b[8..12].copy_from_slice(&self.operation.to_le_bytes());
        b[12..16].copy_from_slice(&self.required_features.to_le_bytes());
        b[16..20].copy_from_slice(&self.payload_bytes.to_le_bytes());
        b[20..22].copy_from_slice(&self.handles.to_le_bytes());
        b[24..32].copy_from_slice(&self.request.to_le_bytes());
        b[32..40].copy_from_slice(&self.route.0.to_le_bytes());
        b[40..].copy_from_slice(&self.policy.0.to_le_bytes());
        b
    }
    pub fn decode(b: &[u8], features: u32) -> Result<Self, u64> {
        if b.len() < 48 || b.len() > 256 {
            return Err(INVALIDARGUMENT);
        }
        let u16at = |i| u16::from_le_bytes(b[i..i + 2].try_into().unwrap());
        let u32at = |i| u32::from_le_bytes(b[i..i + 4].try_into().unwrap());
        let u64at = |i| u64::from_le_bytes(b[i..i + 8].try_into().unwrap());
        if u32at(0) != 0x53444b || u16at(4) != 1 || u32at(12) & !features != 0 {
            return Err(INCOMPATIBLEPROTOCOL);
        }
        if u16at(22) != 0
            || u32at(16) as usize != b.len() - 48
            || u16at(20) > 4
            || !(1..=3).contains(&u32at(8))
        {
            return Err(INVALIDARGUMENT);
        }
        Ok(Self {
            protocol: u32at(0),
            major: u16at(4),
            minor: u16at(6),
            operation: u32at(8),
            required_features: u32at(12),
            payload_bytes: u32at(16),
            handles: u16at(20),
            request: u64at(24),
            route: RouteEpoch(u64at(32)),
            policy: PolicyEpoch(u64at(40)),
        })
    }
}
/// Validate descriptor values before encoding; virtual addresses are copied by
/// the kernel and are neither object identity nor authority.
pub fn map(base: u64, offset: u64, bytes: u64, access: u64) -> Result<[u8; 32], u64> {
    if bytes == 0
        || base % 4096 != 0
        || offset % 4096 != 0
        || bytes % 4096 != 0
        || access > 2
        || base < 65536
        || base.checked_add(bytes).is_none_or(|v| v > 0x80000000)
        || offset.checked_add(bytes).is_none()
    {
        return Err(INVALIDARGUMENT);
    }
    Ok(words([base, offset, bytes, access]))
}
pub fn attachment(handle: Handle, rights: u64, mode: u32) -> Result<[u8; 24], u64> {
    if handle.0 == 0 || rights == 0 || rights & !4095 != 0 || mode > 1 {
        return Err(INVALIDARGUMENT);
    }
    let mut b = [0; 24];
    b[..8].copy_from_slice(&handle.0.to_le_bytes());
    b[8..16].copy_from_slice(&rights.to_le_bytes());
    b[16..20].copy_from_slice(&mode.to_le_bytes());
    Ok(b)
}
pub fn timer(deadline: MonotonicNs, period: u64) -> Result<[u8; 24], u64> {
    if deadline == NEVER {
        return Err(INVALIDARGUMENT);
    }
    Ok(words([deadline.0, period, 0]))
}
pub fn wait_item(handle: Handle, interest: u64, bytes: u64, handles: u64) -> Result<[u8; 32], u64> {
    if handle.0 == 0
        || interest > 3
        || bytes > 256
        || handles > 4
        || (interest != 3 && (bytes != 0 || handles != 0))
    {
        return Err(INVALIDARGUMENT);
    }
    Ok(words([handle.0, interest, bytes, handles]))
}
/// Structural validation only. Object type/rights, actual user mappings, current
/// policy and transfer ownership are validated at the kernel/provider boundary.
pub fn validate(kind: &str, b: &[u8]) -> Result<(), u64> {
    match kind {
        "startup" => {
            Startup::decode(b)?;
        }
        "header" => {
            Header::decode(b, 1)?;
        }
        "service_request" => {
            let w = read_words::<2>(b)?;
            if w[1] != 0 {
                return Err(INVALIDARGUMENT);
            }
        }
        "map" => {
            let w = read_words::<4>(b)?;
            map(w[0], w[1], w[2], w[3])?;
        }
        "attachment" => {
            if b.len() != 24 || b[20..] != [0; 4] {
                return Err(INVALIDARGUMENT);
            }
            let h = u64::from_le_bytes(b[..8].try_into().unwrap());
            let r = u64::from_le_bytes(b[8..16].try_into().unwrap());
            let m = u32::from_le_bytes(b[16..20].try_into().unwrap());
            attachment(Handle(h), r, m)?;
        }
        "timer" => {
            let w = read_words::<3>(b)?;
            if w[2] != 0 {
                return Err(INVALIDARGUMENT);
            }
            timer(MonotonicNs(w[0]), w[1])?;
        }
        "wait_item" => {
            let w = read_words::<4>(b)?;
            wait_item(Handle(w[0]), w[1], w[2], w[3])?;
        }
        "discovery" => {
            let w = read_words::<4>(b)?;
            if w[0] != 0 || w[1] != 4096 {
                return Err(INCOMPATIBLEPROTOCOL);
            }
        }
        "send" => {
            let w = read_words::<4>(b)?;
            if w[1] > 256 || w[3] > 4 {
                return Err(INVALIDARGUMENT);
            }
        }
        "receive" => {
            read_words::<6>(b)?;
        }
        "metadata" => {
            let w = read_words::<8>(b)?;
            if w[0] > 256 || w[1] > 4 || w[6] > 1 || w[7] != 0 {
                return Err(INVALIDARGUMENT);
            }
        }
        "wait" => {
            let w = read_words::<4>(b)?;
            if w[1] == 0 || w[1] > 8 {
                return Err(INVALIDARGUMENT);
            }
        }
        "attempt" => {
            read_words::<3>(b)?;
        }
        "context" => {
            let w = read_words::<5>(b)?;
            if w[4] > 1 {
                return Err(INVALIDARGUMENT);
            }
        }
        "route" => {
            read_words::<20>(b)?;
        }
        _ => return Err(UNSUPPORTED),
    }
    Ok(())
}
