//! Bounded byte parsers and clock arithmetic, independent of hardware access.
use crate::{Range, Region, Result, error};
pub const MAX_TABLE: usize = 65536;
pub fn u32_at(b: &[u8], i: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        b.get(i..i.checked_add(4).ok_or(error("acpi", "length"))?)
            .ok_or(error("acpi", "length"))?
            .try_into()
            .unwrap(),
    ))
}
pub fn u64_at(b: &[u8], i: usize) -> Result<u64> {
    Ok(u64::from_le_bytes(
        b.get(i..i.checked_add(8).ok_or(error("acpi", "length"))?)
            .ok_or(error("acpi", "length"))?
            .try_into()
            .unwrap(),
    ))
}
fn checksum(b: &[u8]) -> Result<()> {
    if b.iter().fold(0u8, |s, x| s.wrapping_add(*x)) != 0 {
        return Err(error("acpi", "checksum"));
    }
    Ok(())
}
pub fn acpi_range(regions: &[Region], base: u64, len: u64) -> Result<Range> {
    let r = Range::new(base, len, "acpi")?.physical("acpi")?;
    // Map whole pages only when every page is firmware-declared ACPI memory.
    let start = base & !4095;
    let end = r.end().checked_add(4095).ok_or(error("acpi", "overflow"))? & !4095;
    if len > MAX_TABLE as u64
        || end <= start
        || !(start..end).step_by(4096).all(|p| {
            regions
                .iter()
                .any(|s| matches!(s.kind, 2 | 3) && s.range.contains(Range { base: p, len: 4096 }))
        })
    {
        return Err(error("acpi", "range"));
    }
    Ok(r)
}
pub fn rsdp(b: &[u8]) -> Result<u64> {
    if b.len() != 36 || &b[..8] != b"RSD PTR " {
        return Err(error("rsdp", "signature"));
    }
    checksum(&b[..20])?;
    if b[15] != 2 || u32_at(b, 20)? != 36 {
        return Err(error("rsdp", "revision"));
    }
    checksum(b)?;
    let p = u64_at(b, 24)?;
    if p == 0 {
        return Err(error("xsdt", "missing"));
    }
    Ok(p)
}
pub fn table_len(header: &[u8]) -> Result<usize> {
    let n = u32_at(header, 4)? as usize;
    if !(36..=MAX_TABLE).contains(&n) {
        return Err(error("acpi", "length"));
    }
    Ok(n)
}
pub fn table<'a>(b: &'a [u8], signature: &[u8; 4]) -> Result<&'a [u8]> {
    if b.len() < 36 || &b[..4] != signature {
        return Err(error("acpi", "signature"));
    }
    if table_len(b)? != b.len() {
        return Err(error("acpi", "length"));
    }
    checksum(b)?;
    Ok(b)
}
pub fn xsdt(b: &[u8], entries: &mut [u64; 128]) -> Result<usize> {
    table(b, b"XSDT")?;
    if (b.len() - 36) % 8 != 0 || (b.len() - 36) / 8 > entries.len() {
        return Err(error("xsdt", "capacity"));
    }
    let n = (b.len() - 36) / 8;
    for i in 0..n {
        entries[i] = u64_at(b, 36 + i * 8)?;
        if entries[i] == 0 || entries[..i].contains(&entries[i]) {
            return Err(error("xsdt", "entry"));
        }
    }
    Ok(n)
}
pub fn madt(b: &[u8], bsp: u8) -> Result<(u64, u64)> {
    table(b, b"APIC")?;
    if b.len() < 44 {
        return Err(error("madt", "length"));
    }
    let base = u32_at(b, 36)? as u64;
    let mut found = false;
    let mut io = None;
    let mut at = 44;
    while at < b.len() {
        let len = *b.get(at + 1).ok_or(error("madt", "length"))? as usize;
        if len < 2 || at + len > b.len() {
            return Err(error("madt", "length"));
        }
        match b[at] {
            0 => {
                if len != 8 {
                    return Err(error("madt", "length"));
                }
                if b[at + 3] == bsp && u32_at(b, at + 4)? & 1 != 0 {
                    if found {
                        return Err(error("madt", "duplicate_cpu"));
                    }
                    found = true;
                }
            }
            1 => {
                if len != 12 || io.is_some() || u32_at(b, at + 8)? != 0 {
                    return Err(error("madt", "unsupported_ioapic"));
                }
                let address = u32_at(b, at + 4)? as u64;
                if address == 0 || address % 4096 != 0 || address == base {
                    return Err(error("madt", "ioapic_address"));
                }
                io = Some(address);
            }
            5 | 9 => return Err(error("madt", "unsupported_topology")),
            _ => {}
        }
        at += len;
    }
    if !found || base == 0 || base % 4096 != 0 {
        return Err(error("madt", "unsupported"));
    }
    Ok((base, io.ok_or(error("madt", "missing_ioapic"))?))
}
pub fn hpet(b: &[u8]) -> Result<(u64, u32)> {
    table(b, b"HPET")?;
    if b.len() != 56
        || b[40] != 0
        || !matches!(b[41], 0 | 64)
        || b[42] != 0
        || b[43] != 0
        || b[52] != 0
    {
        return Err(error("hpet", "unsupported"));
    }
    let base = u64_at(b, 44)?;
    if base == 0 || base % 4096 != 0 {
        return Err(error("hpet", "address"));
    }
    Ok((base, u32_at(b, 36)?))
}
/// UC device pages must be outside all RAM, kernel, module and framebuffer ranges.
pub fn device_range(regions: &[Region], base: u64) -> Result<Range> {
    let r = Range::new(base, 4096, "mmio")?.physical("mmio")?;
    if base == 0
        || base % 4096 != 0
        || r.end() > 1u64 << 32
        || regions.iter().any(|s| s.kind != 1 && s.range.overlaps(r))
    {
        return Err(error("mmio", "range"));
    }
    Ok(r)
}
pub fn period(cap: u64) -> Result<u64> {
    let fs = cap >> 32;
    if cap & (1 << 13) == 0 || cap & 255 == 0 || fs == 0 || fs > 100_000_000 {
        return Err(error("hpet", "capability"));
    }
    Ok(fs)
}
pub fn delta(now: u64, before: u64, bits: u32) -> Result<u64> {
    let d = match bits {
        32 => (now as u32).wrapping_sub(before as u32) as u64,
        64 => now.wrapping_sub(before),
        _ => return Err(error("clock", "width")),
    };
    if d >= 1u64 << (bits - 1) {
        return Err(error("clock", "ambiguous_wrap"));
    }
    Ok(d)
}
pub fn nanos(ticks: u64, fs: u64) -> Result<u64> {
    let n = (ticks as u128)
        .checked_mul(fs as u128)
        .ok_or(error("clock", "overflow"))?
        / 1_000_000;
    u64::try_from(n).map_err(|_| error("clock", "overflow"))
}
pub fn calibration(elapsed: u64, reference: u64, fs: u64) -> Result<(u32, u64)> {
    let ns = nanos(reference, fs)?;
    if elapsed == 0 || elapsed >= u32::MAX as u64 || !(10_000_000..=100_000_000).contains(&ns) {
        return Err(error("timer", "calibration"));
    }
    let initial = u32::try_from((elapsed as u128 * 10_000_000 / ns as u128) as u64)
        .map_err(|_| error("timer", "calibration"))?;
    if initial == 0 {
        return Err(error("timer", "calibration"));
    }
    Ok((
        initial,
        nanos(reference, fs)?
            .checked_mul(initial as u64)
            .ok_or(error("clock", "overflow"))?
            / elapsed,
    ))
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    fn sdt(sig: &[u8; 4], n: usize) -> std::vec::Vec<u8> {
        let mut b = std::vec![0;n];
        b[..4].copy_from_slice(sig);
        b[4..8].copy_from_slice(&(n as u32).to_le_bytes());
        fix(&mut b);
        b
    }
    fn fix(b: &mut [u8]) {
        b[9] = 0;
        b[9] = 0u8.wrapping_sub(b.iter().fold(0u8, |s, x| s.wrapping_add(*x)));
    }
    #[test]
    fn acpi_boundaries_and_checksums() {
        let mut b = sdt(b"XSDT", 44);
        b[36..44].copy_from_slice(&0x1000u64.to_le_bytes());
        fix(&mut b);
        let mut entries = [0; 128];
        assert_eq!(xsdt(&b, &mut entries), Ok(1));
        assert_eq!(entries[0], 0x1000);
        b[20] ^= 1;
        assert_eq!(xsdt(&b, &mut entries), Err(error("acpi", "checksum")));
        for n in [0, 35, 65537] {
            let mut h = [0; 36];
            h[4..8].copy_from_slice(&(n as u32).to_le_bytes());
            assert!(table_len(&h).is_err());
        }
        assert!(xsdt(&sdt(b"XSDT", 37), &mut entries).is_err());
        assert!(xsdt(&sdt(b"XSDT", 36 + 129 * 8), &mut entries).is_err());
        let r = [Region {
            range: Range {
                base: 0x1000,
                len: 8192,
            },
            kind: 2,
        }];
        assert!(acpi_range(&r, 0x1fff, 2).is_ok());
        assert!(acpi_range(&r, 0x2fff, 2).is_err());
        assert!(acpi_range(&r, u64::MAX, 2).is_err());
        assert!(device_range(&r, 0x1000).is_err());
        assert!(device_range(&r, 0xfed00000).is_ok());
        assert!(u32_at(&[], usize::MAX).is_err());
        assert!(u64_at(&[], usize::MAX).is_err());
        assert!(device_range(&[], 0).is_err());
        assert!(device_range(&[], 1u64 << 32).is_err());
        assert!(acpi_range(&r, 0x1000, MAX_TABLE as u64 + 1).is_err());
    }
    #[test]
    fn rsdp_revision_signature_and_both_checksums() {
        let mut b = [0u8; 36];
        b[..8].copy_from_slice(b"RSD PTR ");
        b[15] = 2;
        b[20] = 36;
        b[24..32].copy_from_slice(&0x2000u64.to_le_bytes());
        b[8] = 0u8.wrapping_sub(b[..20].iter().fold(0u8, |s, x| s.wrapping_add(*x)));
        b[32] = 0u8.wrapping_sub(b.iter().fold(0u8, |s, x| s.wrapping_add(*x)));
        assert_eq!(rsdp(&b), Ok(0x2000));
        let mut bad = b;
        bad[16] ^= 1;
        assert!(rsdp(&bad).is_err());
        let mut bad = b;
        bad[33] ^= 1;
        assert!(rsdp(&bad).is_err());
        let mut bad = b;
        bad[15] = 1;
        assert!(rsdp(&bad).is_err());
        let mut bad = b;
        bad[0] = 0;
        assert!(rsdp(&bad).is_err());
        assert!(rsdp(&b[..35]).is_err());
    }
    #[test]
    fn topology_and_hpet_fail_closed() {
        let mut m = sdt(b"APIC", 64);
        m[36..40].copy_from_slice(&0xfee00000u32.to_le_bytes());
        m[44..52].copy_from_slice(&[0, 8, 0, 0, 1, 0, 0, 0]);
        m[52..64].copy_from_slice(&[1, 12, 0, 0, 0, 0, 0xc0, 0xfe, 0, 0, 0, 0]);
        fix(&mut m);
        assert_eq!(madt(&m, 0), Ok((0xfee00000, 0xfec00000)));
        assert!(madt(&m, 1).is_err());
        m[45] = 0;
        fix(&mut m);
        assert!(madt(&m, 0).is_err());
        m[45] = 20;
        fix(&mut m);
        assert!(madt(&m, 0).is_err());
        let mut h = sdt(b"HPET", 56);
        h[41] = 64;
        h[44..52].copy_from_slice(&0xfed00000u64.to_le_bytes());
        fix(&mut h);
        assert_eq!(hpet(&h), Ok((0xfed00000, 0)));
        h[41] = 0;
        fix(&mut h);
        assert!(hpet(&h).is_ok());
        h[41] = 32;
        fix(&mut h);
        assert!(hpet(&h).is_err());
        h[41] = 0;
        h[40] = 1;
        fix(&mut h);
        assert!(hpet(&h).is_err());
        assert!(period(0).is_err());
        assert_eq!(period(10_000_000u64 << 32 | 0x2001), Ok(10_000_000));
    }
    #[test]
    fn clock_wrap_overflow_and_calibration() {
        assert_eq!(delta(3, u32::MAX as u64 - 2, 32), Ok(6));
        assert_eq!(delta(3, u64::MAX - 2, 64), Ok(6));
        assert!(delta(0, 1, 64).is_err());
        assert_eq!(nanos(100, 10_000_000), Ok(1000));
        assert!(nanos(u64::MAX, 100_000_000).is_err());
        assert_eq!(
            calibration(1250000, 2000000, 10_000_000),
            Ok((625000, 10_000_000))
        );
        assert!(calibration(0, 2000000, 10_000_000).is_err());
        assert!(calibration(5, 0, 1).is_err());
    }
}
