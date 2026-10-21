//! Explicit little-endian encoding. No native layouts, pointers, caches or grants.
use super::*;
pub const STATE_VERSION: u16 = 1;
pub const MAX_SNAPSHOT: usize = 16 * 1024 * 1024;
const MAGIC: &[u8; 8] = b"COSDOC01";
struct Writer(Vec<u8>);
impl Writer {
    fn u8(&mut self, n: u8) {
        self.0.push(n);
    }
    fn u16(&mut self, n: u16) {
        self.0.extend(n.to_le_bytes());
    }
    fn u32(&mut self, n: u32) {
        self.0.extend(n.to_le_bytes());
    }
    fn u64(&mut self, n: u64) {
        self.0.extend(n.to_le_bytes());
    }
    fn string(&mut self, s: &str) {
        self.u32(s.len() as u32);
        self.0.extend(s.as_bytes());
    }
    fn frame(&mut self, f: &Frame) {
        self.string(&f.text);
        self.u32(f.selection.anchor);
        self.u32(f.selection.caret);
    }
    fn frames(&mut self, frames: &VecDeque<Frame>) {
        self.u32(frames.len() as u32);
        for f in frames {
            self.frame(f);
        }
    }
}
struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.offset.checked_add(n).ok_or(Error::Bounds)?;
        let bytes = self.bytes.get(self.offset..end).ok_or(Error::Invalid)?;
        self.offset = end;
        Ok(bytes)
    }
    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(
            self.take(2)?.try_into().map_err(|_| Error::Invalid)?,
        ))
    }
    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().map_err(|_| Error::Invalid)?,
        ))
    }
    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().map_err(|_| Error::Invalid)?,
        ))
    }
    fn boolean(&mut self) -> Result<bool> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::Invalid),
        }
    }
    fn string(&mut self, max: usize) -> Result<String> {
        let len = self.u32()? as usize;
        if len > max {
            return Err(Error::Bounds);
        }
        // Check the actual input slice and UTF-8 before allocating.
        Ok(core::str::from_utf8(self.take(len)?)
            .map_err(|_| Error::Invalid)?
            .into())
    }
    fn frame(&mut self) -> Result<Frame> {
        Ok(Frame {
            text: self.string(MAX_TEXT)?,
            selection: Selection::new(self.u32()?, self.u32()?),
        })
    }
    fn frames(&mut self) -> Result<VecDeque<Frame>> {
        let count = self.u32()? as usize;
        if count > HISTORY {
            return Err(Error::Bounds);
        }
        let mut frames = VecDeque::new();
        for _ in 0..count {
            frames.push_back(self.frame()?);
        }
        Ok(frames)
    }
}
impl Model {
    pub fn encode(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut w = Writer(Vec::new());
        w.0.extend(MAGIC);
        w.u16(STATE_VERSION);
        for n in [UNICODE_VERSION.0, UNICODE_VERSION.1, UNICODE_VERSION.2] {
            w.u8(n as u8);
        }
        w.u32(0); // Exact total length, filled after encoding.
        w.u64(self.revision);
        w.u64(self.document.id);
        w.u64(self.document.revision);
        w.frame(&self.document.frame);
        w.string(&self.document.saved);
        w.frames(&self.document.undo);
        w.frames(&self.document.redo);
        w.u32(self.document.scroll);
        w.string(&self.document.query);
        w.u8(match self.preferences.theme {
            Theme::Light => 0,
            Theme::Dark => 1,
            Theme::HighContrast => 2,
        });
        w.u16(self.preferences.text_size);
        w.u8(self.preferences.reduced_motion as u8);
        w.u8(self.preferences.reduced_effects as u8);
        for n in self.session.window {
            w.0.extend(n.to_le_bytes());
        }
        w.string(&self.session.focus);
        w.u32(self.session.file_scroll);
        w.u32(self.session.selected_file);
        if w.0.len() > MAX_SNAPSHOT {
            return Err(Error::Bounds);
        }
        let len = w.0.len() as u32;
        w.0[13..17].copy_from_slice(&len.to_le_bytes());
        Ok(w.0)
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_SNAPSHOT {
            return Err(Error::Bounds);
        }
        let mut r = Reader { bytes, offset: 0 };
        if r.take(8)? != MAGIC {
            return Err(Error::Invalid);
        }
        if r.u16()? != STATE_VERSION {
            return Err(Error::Unsupported);
        }
        if (r.u8()? as u64, r.u8()? as u64, r.u8()? as u64) != UNICODE_VERSION {
            return Err(Error::Unsupported);
        }
        if r.u32()? as usize != bytes.len() {
            return Err(Error::Invalid);
        }
        let revision = r.u64()?;
        let document = Document {
            id: r.u64()?,
            revision: r.u64()?,
            frame: r.frame()?,
            saved: r.string(MAX_TEXT)?,
            undo: r.frames()?,
            redo: r.frames()?,
            scroll: r.u32()?,
            query: r.string(MAX_QUERY)?,
        };
        let theme = match r.u8()? {
            0 => Theme::Light,
            1 => Theme::Dark,
            2 => Theme::HighContrast,
            _ => return Err(Error::Invalid),
        };
        let preferences = Preferences {
            theme,
            text_size: r.u16()?,
            reduced_motion: r.boolean()?,
            reduced_effects: r.boolean()?,
        };
        let mut window = [0; 4];
        for n in &mut window {
            *n = i32::from_le_bytes(r.take(4)?.try_into().map_err(|_| Error::Invalid)?);
        }
        let session = Session {
            window,
            focus: r.string(64)?,
            file_scroll: r.u32()?,
            selected_file: r.u32()?,
        };
        if r.offset != bytes.len() {
            return Err(Error::Invalid);
        }
        let model = Self {
            revision,
            document,
            preferences,
            session,
        };
        model.validate()?;
        Ok(model)
    }
}
