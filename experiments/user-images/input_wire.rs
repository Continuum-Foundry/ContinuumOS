//! Explicit experimental CIN0 bytes; no native-layout serialization.
pub const KEY: u16 = 1;
pub const TEXT: u16 = 2;
pub const MOTION: u16 = 3;
pub const BUTTON: u16 = 4;
pub const CANCEL: u16 = 5;
pub const FOCUS: u16 = 6;
pub const WIRE: usize = 96;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Event {
    pub sequence: u64,
    pub time: u64,
    pub epoch: u64,
    pub surface: u64,
    pub scene: u64,
    pub output: u64,
    pub focus: u64,
    pub kind: u16,
    pub code: u16,
    pub value: i32,
    pub x: i32,
    pub y: i32,
    pub modifiers: u32,
    pub text: u32,
}
impl Event {
    pub const EMPTY: Self = Self {
        sequence: 0,
        time: 0,
        epoch: 1,
        surface: 0,
        scene: 0,
        output: 1,
        focus: 0,
        kind: 0,
        code: 0,
        value: 0,
        x: 0,
        y: 0,
        modifiers: 0,
        text: 0,
    };
    pub fn encode(self) -> [u8; WIRE] {
        let mut b = [0; WIRE];
        b[..4].copy_from_slice(b"CIN0");
        b[4..6].copy_from_slice(&0u16.to_le_bytes());
        b[6..8].copy_from_slice(&(WIRE as u16).to_le_bytes());
        for (n, v) in [
            self.sequence,
            self.time,
            self.epoch,
            self.surface,
            self.scene,
            self.output,
            self.focus,
        ]
        .into_iter()
        .enumerate()
        {
            b[8 + n * 8..16 + n * 8].copy_from_slice(&v.to_le_bytes());
        }
        b[64..66].copy_from_slice(&self.kind.to_le_bytes());
        b[66..68].copy_from_slice(&self.code.to_le_bytes());
        for (n, v) in [
            self.value as u32,
            self.x as u32,
            self.y as u32,
            self.modifiers,
            self.text,
        ]
        .into_iter()
        .enumerate()
        {
            b[68 + n * 4..72 + n * 4].copy_from_slice(&v.to_le_bytes());
        }
        b
    }
    pub fn decode(b: &[u8]) -> Result<Self, ()> {
        if b.len() != WIRE || &b[..8] != b"CIN0\0\0\x60\0" || b[88..] != [0; 8] {
            return Err(());
        }
        let w = |n| u64::from_le_bytes(b[n..n + 8].try_into().unwrap());
        let d = |n| u32::from_le_bytes(b[n..n + 4].try_into().unwrap());
        let e = Self {
            sequence: w(8),
            time: w(16),
            epoch: w(24),
            surface: w(32),
            scene: w(40),
            output: w(48),
            focus: w(56),
            kind: u16::from_le_bytes(b[64..66].try_into().unwrap()),
            code: u16::from_le_bytes(b[66..68].try_into().unwrap()),
            value: d(68) as i32,
            x: d(72) as i32,
            y: d(76) as i32,
            modifiers: d(80),
            text: d(84),
        };
        if e.sequence == 0
            || e.epoch == 0
            || e.surface == 0
            || e.output == 0
            || !(KEY..=FOCUS).contains(&e.kind)
            || e.modifiers & !7 != 0
            || e.text != 0 && char::from_u32(e.text).is_none()
        {
            return Err(());
        }
        Ok(e)
    }
}
