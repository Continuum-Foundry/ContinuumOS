//! Explicit little-endian draft service envelope; no native layouts or allocation.
pub const HEADER: usize = 48;
pub const INLINE: usize = 4096;
pub const ATTACHMENTS: usize = 8;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Length,
    Reserved,
    Protocol,
    Features,
    Handles,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Envelope {
    pub protocol: u32,
    pub major: u16,
    pub minor: u16,
    pub operation: u32,
    pub required_features: u32,
    pub request_id: u64,
    pub route_epoch: u64,
    pub policy_epoch: u64,
}
fn u16_at(b: &[u8], i: usize) -> u16 {
    u16::from_le_bytes(b[i..i + 2].try_into().unwrap())
}
fn u32_at(b: &[u8], i: usize) -> u32 {
    u32::from_le_bytes(b[i..i + 4].try_into().unwrap())
}
fn u64_at(b: &[u8], i: usize) -> u64 {
    u64::from_le_bytes(b[i..i + 8].try_into().unwrap())
}
impl Envelope {
    pub fn encode(self, payload: &[u8], handles: usize, out: &mut [u8]) -> Result<usize, Error> {
        let length = HEADER
            .checked_add(payload.len())
            .filter(|n| *n <= INLINE && *n <= out.len())
            .ok_or(Error::Length)?;
        if handles > ATTACHMENTS {
            return Err(Error::Handles);
        }
        out[0..4].copy_from_slice(&self.protocol.to_le_bytes());
        out[4..6].copy_from_slice(&self.major.to_le_bytes());
        out[6..8].copy_from_slice(&self.minor.to_le_bytes());
        out[8..12].copy_from_slice(&self.operation.to_le_bytes());
        out[12..16].copy_from_slice(&self.required_features.to_le_bytes());
        out[16..20].copy_from_slice(&(payload.len() as u32).to_le_bytes());
        out[20..22].copy_from_slice(&(handles as u16).to_le_bytes());
        out[22..24].fill(0);
        out[24..32].copy_from_slice(&self.request_id.to_le_bytes());
        out[32..40].copy_from_slice(&self.route_epoch.to_le_bytes());
        out[40..48].copy_from_slice(&self.policy_epoch.to_le_bytes());
        out[HEADER..length].copy_from_slice(payload);
        Ok(length)
    }
    pub fn decode(
        bytes: &[u8],
        actual_handles: usize,
        protocol: u32,
        major: u16,
        maximum_minor: u16,
        features: u32,
    ) -> Result<(Self, &[u8]), Error> {
        if bytes.len() < HEADER || bytes.len() > INLINE {
            return Err(Error::Length);
        }
        if u32_at(bytes, 16) as usize != bytes.len() - HEADER {
            return Err(Error::Length);
        }
        if u16_at(bytes, 22) != 0 {
            return Err(Error::Reserved);
        }
        if actual_handles > ATTACHMENTS || u16_at(bytes, 20) as usize != actual_handles {
            return Err(Error::Handles);
        }
        let e = Self {
            protocol: u32_at(bytes, 0),
            major: u16_at(bytes, 4),
            minor: u16_at(bytes, 6),
            operation: u32_at(bytes, 8),
            required_features: u32_at(bytes, 12),
            request_id: u64_at(bytes, 24),
            route_epoch: u64_at(bytes, 32),
            policy_epoch: u64_at(bytes, 40),
        };
        if e.protocol != protocol || e.major != major || e.minor > maximum_minor {
            return Err(Error::Protocol);
        }
        if e.required_features & !features != 0 {
            return Err(Error::Features);
        }
        Ok((e, &bytes[HEADER..]))
    }
}
