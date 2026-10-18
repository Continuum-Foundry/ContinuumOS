use serde_json::{Value, json};

fn integer(bytes: &[u8], offset: usize, size: usize) -> Result<u64, String> {
    let part = bytes
        .get(offset..offset.checked_add(size).ok_or("offset overflow")?)
        .ok_or("truncated ELF")?;
    let mut value = 0;
    for (index, byte) in part.iter().enumerate() {
        value |= u64::from(*byte) << (index * 8);
    }
    Ok(value)
}
fn range(bytes: &[u8], offset: u64, size: u64) -> Result<(), String> {
    let end = offset.checked_add(size).ok_or("ELF range overflow")?;
    if end > bytes.len() as u64 {
        return Err("ELF range outside file".into());
    }
    Ok(())
}

// A deliberately narrow validator for our two static ELF64 probes, not a guest loader.
pub fn inspect(bytes: &[u8], component: &str) -> Result<Value, String> {
    if bytes.get(..7) != Some(b"\x7fELF\x02\x01\x01") {
        return Err("Expected ELF64 little-endian version 1".into());
    }
    let read = |offset, size| integer(bytes, offset, size);
    if read(16, 2)? != 2 || read(18, 2)? != 62 || read(20, 4)? != 1 || read(52, 2)? != 64 {
        return Err("Expected x86-64 ET_EXEC ELF header".into());
    }
    let entry = read(24, 8)?;
    let expected = match component {
        "kernel-link" => 0xffff_ffff_8000_0000,
        "userland-link" => 0x400000,
        _ => return Err("Unknown component".into()),
    };
    if entry != expected {
        return Err(format!("Unexpected {component} entry: {entry:#x}"));
    }
    let phoff = read(32, 8)?;
    let phsize = read(54, 2)?;
    let phnum = read(56, 2)?;
    if phsize != 56 || phnum == 0 || phnum > 64 {
        return Err("Invalid program header table".into());
    }
    range(bytes, phoff, phsize * phnum)?;
    let mut segments = Vec::new();
    let mut entry_executable = false;
    let mut memory_ranges = Vec::new();
    for index in 0..phnum {
        let p = (phoff + index * phsize) as usize;
        let kind = read(p, 4)?;
        let flags = read(p + 4, 4)?;
        if [2, 3, 7].contains(&kind) {
            return Err("Dynamic linking, interpreter or TLS segment found".into());
        }
        if kind == 0x6474e551 && flags & 1 != 0 {
            return Err("Executable stack found".into());
        }
        if kind != 1 {
            continue;
        }
        let offset = read(p + 8, 8)?;
        let address = read(p + 16, 8)?;
        let file_size = read(p + 32, 8)?;
        let memory_size = read(p + 40, 8)?;
        let alignment = read(p + 48, 8)?;
        if flags & 3 == 3 {
            return Err("Writable executable load segment found".into());
        }
        if flags & 4 == 0 || flags & !7 != 0 || file_size > memory_size {
            return Err("Invalid load segment flags/sizes".into());
        }
        if alignment > 1
            && (!alignment.is_power_of_two() || offset % alignment != address % alignment)
        {
            return Err("Invalid load alignment".into());
        }
        range(bytes, offset, file_size)?;
        let end = address
            .checked_add(memory_size)
            .ok_or("Virtual range overflow")?;
        if memory_size > 0 {
            if memory_ranges
                .iter()
                .any(|(start, stop)| address < *stop && end > *start)
            {
                return Err("Overlapping load segments".into());
            }
            memory_ranges.push((address, end));
        }
        if flags & 1 != 0 && entry >= address && entry - address < file_size {
            entry_executable = true;
        }
        segments.push(json!({"address":format!("{address:#x}"),"offset":offset,"file_size":file_size,"memory_size":memory_size,"flags":flags,"alignment":alignment}));
    }
    if !entry_executable {
        return Err("Entry is not in file-backed executable memory".into());
    }
    let shoff = read(40, 8)?;
    let shsize = read(58, 2)?;
    let shnum = read(60, 2)?;
    if shsize != 64 || shnum == 0 {
        return Err("Missing section/symbol evidence".into());
    }
    range(bytes, shoff, shsize * shnum)?;
    let mut symbol_tables = 0;
    for index in 0..shnum {
        let s = (shoff + index * shsize) as usize;
        let kind = read(s + 4, 4)?;
        let flags = read(s + 8, 8)?;
        let offset = read(s + 24, 8)?;
        let size = read(s + 32, 8)?;
        if kind != 8 {
            range(bytes, offset, size)?;
        }
        if flags & 7 == 7 {
            return Err("Writable executable allocated section".into());
        }
        if [4, 9].contains(&kind) && size != 0 {
            return Err("Remaining relocation section found".into());
        }
        if [6, 11].contains(&kind) {
            return Err("Dynamic section/symbols found".into());
        }
        if kind == 2 {
            symbol_tables += 1;
            if read(s + 56, 8)? != 24 || size % 24 != 0 {
                return Err("Invalid symbol table".into());
            }
            for symbol in 0..size / 24 {
                let at = (offset + symbol * 24) as usize;
                if read(at, 4)? != 0 && read(at + 6, 2)? == 0 {
                    return Err("Unresolved named symbol found".into());
                }
            }
        }
    }
    if symbol_tables == 0 {
        return Err("No static symbol table for unresolved-symbol inspection".into());
    }
    Ok(
        json!({"format":"ELF64-LE","machine":"x86_64","type":"ET_EXEC","entry":format!("{entry:#x}"),"segments":segments,"dynamic_linking":false,"unresolved_symbols":0,"relocations":0,"executed":false}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn put(b: &mut [u8], p: usize, value: u64, n: usize) {
        b[p..p + n].copy_from_slice(&value.to_le_bytes()[..n]);
    }
    fn fixture() -> Vec<u8> {
        let mut b = vec![0; 512];
        b[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
        for (p, v, n) in [
            (16, 2, 2),
            (18, 62, 2),
            (20, 1, 4),
            (24, 0x400000, 8),
            (32, 64, 8),
            (40, 256, 8),
            (52, 64, 2),
            (54, 56, 2),
            (56, 1, 2),
            (58, 64, 2),
            (60, 2, 2),
            (64, 1, 4),
            (68, 5, 4),
            (72, 128, 8),
            (80, 0x400000, 8),
            (96, 1, 8),
            (104, 1, 8),
            (112, 1, 8),
            (324, 2, 4),
            (344, 400, 8),
            (352, 24, 8),
            (376, 24, 8),
        ] {
            put(&mut b, p, v, n);
        }
        b
    }
    #[test]
    fn valid_static_probe() {
        assert!(inspect(&fixture(), "userland-link").is_ok());
    }
    #[test]
    fn truncation_and_overflow() {
        let good = fixture();
        for size in 0..good.len() {
            let _ = inspect(&good[..size], "userland-link");
        }
        let mut b = good;
        put(&mut b, 32, u64::MAX, 8);
        assert!(inspect(&b, "userland-link").is_err());
    }
    #[test]
    fn rejects_unsafe_or_host_linked_images() {
        for (p, v, n) in [
            (4, 1, 1),
            (5, 2, 1),
            (18, 3, 2),
            (68, 7, 4),
            (64, 2, 4),
            (64, 3, 4),
            (64, 7, 4),
            (104, 0, 8),
            (112, 3, 8),
            (324, 4, 4),
            (324, 6, 4),
            (400, 1, 4),
            (24, 0x500000, 8),
        ] {
            let mut b = fixture();
            put(&mut b, p, v, n);
            assert!(
                inspect(&b, "userland-link").is_err(),
                "accepted mutation at {p}"
            );
        }
    }
}
