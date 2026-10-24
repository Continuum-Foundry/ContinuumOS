//! Bounded hosted text experiment. Only allowlisted fonts are parsed. No guest runtime.
#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::{collections::VecDeque, string::String, vec, vec::Vec};
use core::ops::Range;
use rustybuzz::{Direction, UnicodeBuffer};
use sha2::{Digest, Sha256};
use unicode_bidi::BidiInfo;
use unicode_script::{Script, UnicodeScript};
use unicode_segmentation::UnicodeSegmentation;

mod composition;
pub use composition::Composition;
mod pins;
pub use pins::{FONT_HASHES, FONT_NAMES};

pub const MAX_TEXT: usize = 4096;
pub const MAX_GLYPHS: usize = 8192;
pub const MAX_LINES: usize = 256;
pub const MAX_FONT_BYTES: usize = 24 * 1024 * 1024;
pub const MAX_TABLES: usize = 128;
pub const MAX_TABLE_BYTES: usize = 20 * 1024 * 1024;
pub const MAX_CACHE_BYTES: usize = 1024 * 1024;
pub const MAX_CACHE_ENTRIES: usize = 128;
pub const MAX_LAYOUT_CACHE_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_LAYOUT_CACHE_ENTRIES: usize = 128;
pub const MAX_BITMAP: usize = 256 * 256;
pub const MAX_SHAPE_WORK: usize = 131072;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Bounds,
    InvalidFont,
    UnpinnedFont,
    Unsupported,
    InvalidSelection,
    Conflict,
    Allocation,
}
pub type Result<T> = core::result::Result<T, Error>;
pub fn boundaries(text: &str) -> Vec<usize> {
    text.grapheme_indices(true)
        .map(|(i, _)| i)
        .chain(core::iter::once(text.len()))
        .collect()
}
pub fn is_boundary(text: &str, at: usize) -> bool {
    at == text.len() || text.grapheme_indices(true).any(|(i, _)| i == at)
}
pub fn words(text: &str) -> Vec<Range<usize>> {
    text.unicode_word_indices()
        .map(|(i, s)| i..i + s.len())
        .collect()
}
pub fn validate_text(text: &str) -> Result<()> {
    if text.len() > MAX_TEXT || text.bytes().filter(|b| *b == b'\n').count() >= MAX_LINES {
        return Err(Error::Bounds);
    }
    if text.chars().any(|c| {
        (c.is_control() && c != '\n')
            || matches!(c,'\u{2028}'|'\u{2029}'|'\u{202a}'..='\u{202e}'|'\u{2066}'..='\u{2069}')
    }) {
        return Err(Error::Unsupported);
    }
    Ok(())
}

/// Shallow SFNT bounds check before any third-party parser; not a font sanitizer.
pub fn preflight(bytes: &[u8]) -> Result<()> {
    if bytes.len() > MAX_FONT_BYTES {
        return Err(Error::Bounds);
    }
    if bytes.len() < 12 || !matches!(&bytes[..4], b"\0\x01\0\0" | b"OTTO") {
        return Err(Error::InvalidFont);
    }
    let count = u16::from_be_bytes([bytes[4], bytes[5]]) as usize;
    if count == 0 || count > MAX_TABLES || bytes.len() < 12 + count * 16 {
        return Err(Error::InvalidFont);
    }
    for table in bytes[12..12 + count * 16].chunks_exact(16) {
        let start = u32::from_be_bytes(table[8..12].try_into().unwrap()) as usize;
        let len = u32::from_be_bytes(table[12..16].try_into().unwrap()) as usize;
        if len > MAX_TABLE_BYTES || start.checked_add(len).is_none_or(|end| end > bytes.len()) {
            return Err(Error::InvalidFont);
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub mono: bool,
    pub size: u16,
    pub scale: u16,
    pub width: u16,
    pub ligatures: bool,
}
impl Config {
    pub fn validate(self) -> Result<()> {
        if !(8..=48).contains(&self.size)
            || ![100, 150, 200].contains(&self.scale)
            || self.width == 0
            || self.width > 4096
        {
            return Err(Error::Bounds);
        }
        Ok(())
    }
    pub fn pixels(self) -> f32 {
        self.size as f32 * self.scale as f32 / 100.0
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Glyph {
    pub font: usize,
    pub id: u16,
    pub cluster: Range<usize>,
    pub x: f32,
    pub advance: f32,
    pub offset: [f32; 2],
    pub rtl: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Caret {
    pub byte: usize,
    pub x: f32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub bytes: Range<usize>,
    pub glyphs: Vec<Glyph>,
    pub carets: Vec<Caret>,
    pub width: f32,
}
impl Line {
    /// Nearest visual stop, stable tie to smaller logical offset. Dual bidi positions survive.
    pub fn hit(&self, x: f32) -> Result<usize> {
        if !x.is_finite() {
            return Err(Error::Bounds);
        }
        self.carets
            .iter()
            .min_by(|a, b| {
                (a.x - x)
                    .abs()
                    .total_cmp(&(b.x - x).abs())
                    .then(a.byte.cmp(&b.byte))
            })
            .map(|c| c.byte)
            .ok_or(Error::InvalidSelection)
    }
    pub fn selection(&self, range: Range<usize>) -> Result<Vec<[f32; 2]>> {
        if range.start > range.end
            || ![range.start, range.end]
                .iter()
                .all(|byte| self.carets.iter().any(|c| c.byte == *byte))
        {
            return Err(Error::InvalidSelection);
        }
        // Aggregate marks/ligature glyphs into one visual cluster before finding endpoints.
        let mut rects = Vec::new();
        let mut i = 0;
        while i < self.glyphs.len() {
            let g = &self.glyphs[i];
            let mut end = i + 1;
            while end < self.glyphs.len()
                && self.glyphs[end].font == g.font
                && self.glyphs[end].cluster == g.cluster
            {
                end += 1;
            }
            let width = self.glyphs[i..end].iter().map(|g| g.advance).sum::<f32>();
            let a = range.start.max(g.cluster.start);
            let b = range.end.min(g.cluster.end);
            if a < b {
                let at = |byte| {
                    self.carets
                        .iter()
                        .find(|c| c.byte == byte && c.x >= g.x - 0.01 && c.x <= g.x + width + 0.01)
                        .map(|c| c.x)
                        .ok_or(Error::InvalidSelection)
                };
                let x1 = at(a)?;
                let x2 = at(b)?;
                rects.push([x1.min(x2), x1.max(x2)]);
            }
            i = end;
        }
        Ok(rects)
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Layout {
    pub lines: Vec<Line>,
    pub missing: Vec<Range<usize>>,
    pub shape_work: usize,
    /// Document-wide boundaries; shaping clusters can split an Indic/emoji grapheme.
    pub graphemes: Vec<usize>,
}

/// Visual affinity is a line/stop, because a bidi boundary can have two x positions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VisualCaret {
    pub line: usize,
    pub stop: usize,
    pub byte: usize,
    pub x: f32,
}
impl Layout {
    pub fn visual_stops(&self, line: usize) -> Result<Vec<Caret>> {
        let mut stops = self.lines.get(line).ok_or(Error::Bounds)?.carets.clone();
        stops.retain(|c| self.graphemes.contains(&c.byte));
        stops.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.byte.cmp(&b.byte)));
        stops.dedup_by(|a, b| a.byte == b.byte && (a.x - b.x).abs() < 0.001);
        Ok(stops)
    }
    pub fn visual_hit(&self, line: usize, x: f32) -> Result<VisualCaret> {
        if !x.is_finite() {
            return Err(Error::Bounds);
        }
        let stops = self.visual_stops(line)?;
        let (stop, c) = stops
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                (a.x - x)
                    .abs()
                    .total_cmp(&(b.x - x).abs())
                    .then(a.byte.cmp(&b.byte))
            })
            .ok_or(Error::InvalidSelection)?;
        Ok(VisualCaret {
            line,
            stop,
            byte: c.byte,
            x: c.x,
        })
    }
    pub fn visual_move(
        &self,
        byte: usize,
        affinity: Option<VisualCaret>,
        usage: u16,
    ) -> Result<VisualCaret> {
        let current = affinity.filter(|a| {
            a.byte == byte
                && self.visual_stops(a.line).is_ok_and(|s| {
                    s.get(a.stop)
                        .is_some_and(|c| c.byte == a.byte && c.x == a.x)
                })
        });
        let current = match current {
            Some(c) => c,
            None => {
                let mut found = None;
                for line in 0..self.lines.len() {
                    for (stop, c) in self.visual_stops(line)?.iter().enumerate() {
                        if c.byte == byte {
                            found = Some(VisualCaret {
                                line,
                                stop,
                                byte,
                                x: c.x,
                            });
                            break;
                        }
                    }
                    if found.is_some() {
                        break;
                    }
                }
                found.ok_or(Error::InvalidSelection)?
            }
        };
        let mut line = current.line;
        let mut stop = current.stop;
        let stops = self.visual_stops(line)?;
        match usage {
            79 => {
                if stop + 1 < stops.len() {
                    stop += 1;
                } else if line + 1 < self.lines.len() {
                    line += 1;
                    stop = 0;
                }
            }
            80 => {
                if stop > 0 {
                    stop -= 1;
                } else if line > 0 {
                    line -= 1;
                    stop = self.visual_stops(line)?.len().saturating_sub(1);
                }
            }
            81 | 82 => {
                line = if usage == 81 {
                    (line + 1).min(self.lines.len() - 1)
                } else {
                    line.saturating_sub(1)
                };
                return self.visual_hit(line, current.x);
            }
            74 => stop = 0,
            77 => stop = stops.len().saturating_sub(1),
            _ => return Err(Error::Unsupported),
        }
        let c = self
            .visual_stops(line)?
            .get(stop)
            .cloned()
            .ok_or(Error::InvalidSelection)?;
        Ok(VisualCaret {
            line,
            stop,
            byte: c.byte,
            x: c.x,
        })
    }
}

/// Borrowed, immutable raw font inputs; raster faces are lazily decoded and kept outside bitmap LRU.
pub struct Engine<'a> {
    bytes: Vec<&'a [u8]>,
    faces: Vec<rustybuzz::Face<'a>>,
    raster: Vec<Option<fontdue::Font>>,
    cache: VecDeque<(RasterKey, Bitmap)>,
    cache_bytes: usize,
    layouts: VecDeque<(String, Config, Layout)>,
    layout_hits: u64,
    layout_misses: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RasterKey {
    font: usize,
    glyph: u16,
    size: u16,
    scale: u16,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Bitmap {
    pub width: usize,
    pub height: usize,
    pub xmin: i32,
    pub ymin: i32,
    pub pixels: Vec<u8>,
}
impl<'a> Engine<'a> {
    /// Exact six-font inventory, face zero, no font discovery/variation/custom fonts.
    pub fn new(fonts: &[&'a [u8]]) -> Result<Self> {
        if fonts.len() != FONT_HASHES.len() {
            return Err(Error::UnpinnedFont);
        }
        let mut faces = Vec::new();
        faces
            .try_reserve(fonts.len())
            .map_err(|_| Error::Allocation)?;
        for (i, bytes) in fonts.iter().enumerate() {
            preflight(bytes)?;
            let digest: [u8; 32] = Sha256::digest(bytes).into();
            if digest != FONT_HASHES[i] {
                return Err(Error::UnpinnedFont);
            }
            let face = rustybuzz::Face::from_slice(bytes, 0).ok_or(Error::InvalidFont)?;
            if face.number_of_glyphs() == 0 {
                return Err(Error::InvalidFont);
            }
            faces.push(face);
        }
        Ok(Self {
            bytes: fonts.to_vec(),
            faces,
            raster: (0..fonts.len()).map(|_| None).collect(),
            cache: VecDeque::new(),
            cache_bytes: 0,
            layouts: VecDeque::new(),
            layout_hits: 0,
            layout_misses: 0,
        })
    }
    pub fn cache_usage(&self) -> (usize, usize) {
        (self.cache.len(), self.cache_bytes)
    }
    /// Exact text + all shaping/layout parameters; immutable pinned fonts belong
    /// to this engine. This selected-path cache never keys by generation or a hash
    /// alone. Eviction drops owned values and cannot invalidate returned clones.
    pub fn layout_cached(&mut self, text: &str, cfg: Config) -> Result<Layout> {
        cfg.validate()?;
        validate_text(text)?;
        if let Some(i) = self
            .layouts
            .iter()
            .position(|(key, c, _)| key == text && *c == cfg)
        {
            self.layout_hits += 1;
            let item = self.layouts.remove(i).unwrap();
            let mut result = item.2.clone();
            result.shape_work = 0;
            self.layouts.push_back(item);
            return Ok(result);
        }
        self.layout_misses += 1;
        let result = self.layout(text, cfg)?;
        let estimate = Self::layout_bytes(&result) + text.len();
        if estimate + core::mem::size_of::<(String, Config, Layout)>() * MAX_LAYOUT_CACHE_ENTRIES
            <= MAX_LAYOUT_CACHE_BYTES
        {
            if self.layouts.capacity() == 0 {
                self.layouts.reserve_exact(MAX_LAYOUT_CACHE_ENTRIES);
            }
            while self.layouts.len() >= MAX_LAYOUT_CACHE_ENTRIES
                || self.layout_cache_usage().1 + estimate > MAX_LAYOUT_CACHE_BYTES
            {
                if self.layouts.pop_front().is_none() {
                    break;
                }
            }
            self.layouts
                .push_back((String::from(text), cfg, result.clone()));
        }
        Ok(result)
    }
    fn layout_bytes(l: &Layout) -> usize {
        l.lines.capacity() * core::mem::size_of::<Line>()
            + l.missing.capacity() * core::mem::size_of::<Range<usize>>()
            + l.graphemes.capacity() * core::mem::size_of::<usize>()
            + l.lines
                .iter()
                .map(|line| {
                    line.glyphs.capacity() * core::mem::size_of::<Glyph>()
                        + line.carets.capacity() * core::mem::size_of::<Caret>()
                })
                .sum::<usize>()
    }
    pub fn layout_cache_usage(&self) -> (usize, usize, u64, u64) {
        (
            self.layouts.len(),
            self.layouts.capacity() * core::mem::size_of::<(String, Config, Layout)>()
                + self
                    .layouts
                    .iter()
                    .map(|(text, _, l)| text.capacity() + Self::layout_bytes(l))
                    .sum::<usize>(),
            self.layout_hits,
            self.layout_misses,
        )
    }
    fn font_for(&self, grapheme: &str, mono: bool) -> (usize, bool) {
        // A missing cluster is wholly tofu; never split a ZWJ/combining sequence between faces.
        let order = if mono {
            [5, 0, 1, 2, 3, 4]
        } else {
            [0, 1, 2, 3, 4, 5]
        };
        order
            .into_iter()
            .find(|i| {
                let face = &self.faces[*i];
                grapheme
                    .chars()
                    .filter(|c| !matches!(*c, '\u{200d}' | '\u{200c}' | '\u{fe0e}' | '\u{fe0f}'))
                    .all(|c| face.glyph_index(c).is_some())
            })
            .map(|i| (i, false))
            .unwrap_or((0, true))
    }
    fn shape_line(
        &self,
        text: &str,
        bidi: &BidiInfo<'_>,
        para: &unicode_bidi::ParagraphInfo,
        range: Range<usize>,
        cfg: Config,
        work: &mut usize,
    ) -> Result<Line> {
        *work = work.checked_add(range.len()).ok_or(Error::Bounds)?;
        if *work > MAX_SHAPE_WORK {
            return Err(Error::Bounds);
        }
        let (levels, runs) = bidi.visual_runs(para, range.clone());
        let mut glyphs = Vec::new();
        let mut x = 0.0;
        for run in runs {
            let rtl = levels[run.start].is_rtl();
            let mut pieces: Vec<(Range<usize>, usize, Script)> = Vec::new();
            let mut inherited = Script::Common;
            for (i, g) in text[run.clone()].grapheme_indices(true) {
                let start = run.start + i;
                let font = self.font_for(g, cfg.mono).0;
                let script = g
                    .chars()
                    .map(|c| c.script())
                    .find(|s| !matches!(s, Script::Common | Script::Inherited))
                    .unwrap_or(inherited);
                inherited = script;
                if let Some((r, f, s)) = pieces.last_mut() {
                    if *f == font && *s == script {
                        r.end = start + g.len();
                        continue;
                    }
                }
                pieces.push((start..start + g.len(), font, script));
            }
            if rtl {
                pieces.reverse();
            }
            for (piece, font, _) in pieces {
                let mut buffer = UnicodeBuffer::new();
                buffer.push_str(&text[piece.clone()]);
                buffer.set_direction(if rtl {
                    Direction::RightToLeft
                } else {
                    Direction::LeftToRight
                });
                buffer.guess_segment_properties();
                buffer.set_cluster_level(rustybuzz::BufferClusterLevel::MonotoneGraphemes);
                let features = if cfg.ligatures {
                    Vec::new()
                } else {
                    vec!["liga=0".parse().unwrap(), "clig=0".parse().unwrap()]
                };
                let shaped = rustybuzz::shape(&self.faces[font], &features, buffer);
                if glyphs.len() + shaped.len() > MAX_GLYPHS {
                    return Err(Error::Bounds);
                }
                let factor = cfg.pixels() / self.faces[font].units_per_em() as f32;
                let mut clusters: Vec<usize> = shaped
                    .glyph_infos()
                    .iter()
                    .map(|g| piece.start + g.cluster as usize)
                    .collect();
                clusters.push(piece.end);
                clusters.sort_unstable();
                clusters.dedup();
                for (info, pos) in shaped.glyph_infos().iter().zip(shaped.glyph_positions()) {
                    let start = piece.start + info.cluster as usize;
                    let end = *clusters
                        .iter()
                        .find(|end| **end > start)
                        .ok_or(Error::InvalidSelection)?;
                    let advance = pos.x_advance as f32 * factor;
                    if advance < 0.0 {
                        return Err(Error::Unsupported);
                    }
                    glyphs.push(Glyph {
                        font,
                        id: info.glyph_id as u16,
                        cluster: start..end,
                        x,
                        advance,
                        offset: [pos.x_offset as f32 * factor, pos.y_offset as f32 * factor],
                        rtl,
                    });
                    x += advance;
                }
            }
        }
        let mut carets = Vec::new();
        // Aggregate glyphs that belong to a shaping cluster (marks may have zero advance).
        let mut i = 0;
        while i < glyphs.len() {
            let g = &glyphs[i];
            let mut end = i + 1;
            while end < glyphs.len()
                && glyphs[end].font == g.font
                && glyphs[end].cluster == g.cluster
            {
                end += 1;
            }
            let width = glyphs[i..end].iter().map(|g| g.advance).sum::<f32>();
            let stops = boundaries(&text[g.cluster.clone()]);
            let n = stops.len() - 1;
            for (index, byte) in stops.into_iter().enumerate() {
                let fraction = index as f32 / n as f32;
                carets.push(Caret {
                    byte: g.cluster.start + byte,
                    x: g.x + width * if g.rtl { 1.0 - fraction } else { fraction },
                });
            }
            i = end;
        }
        if carets.is_empty() {
            carets.push(Caret {
                byte: range.start,
                x: 0.0,
            });
        }
        carets.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.byte.cmp(&b.byte)));
        carets.dedup_by(|a, b| a.byte == b.byte && (a.x - b.x).abs() < 0.001);
        Ok(Line {
            bytes: range,
            glyphs,
            carets,
            width: x,
        })
    }
    pub fn layout(&self, text: &str, cfg: Config) -> Result<Layout> {
        cfg.validate()?;
        validate_text(text)?;
        if text.len() > MAX_TEXT {
            return Err(Error::Bounds);
        }
        // Rendering control codes and bidi isolates/overrides need additional caret policy.
        if text.chars().any(|c| {
            (c.is_control() && c != '\n')
                || matches!(c,'\u{2028}'|'\u{2029}'|'\u{202a}'..='\u{202e}'|'\u{2066}'..='\u{2069}')
        }) {
            return Err(Error::Unsupported);
        }
        let mut result = Layout {
            lines: Vec::new(),
            missing: Vec::new(),
            shape_work: 0,
            graphemes: boundaries(text),
        };
        for (i, g) in text.grapheme_indices(true) {
            if g != "\n" && self.font_for(g, cfg.mono).1 {
                result.missing.push(i..i + g.len());
            }
        }
        let bidi = BidiInfo::new(text, None);
        for para in &bidi.paragraphs {
            let end = if text[para.range.clone()].ends_with('\n') {
                para.range.end - 1
            } else {
                para.range.end
            };
            let mut start = para.range.start;
            if start == end {
                result.lines.push(Line {
                    bytes: start..end,
                    glyphs: Vec::new(),
                    carets: vec![Caret {
                        byte: start,
                        x: 0.0,
                    }],
                    width: 0.0,
                });
            }
            while start < end {
                if result.lines.len() >= MAX_LINES {
                    return Err(Error::Bounds);
                }
                let remaining =
                    self.shape_line(text, &bidi, para, start..end, cfg, &mut result.shape_work)?;
                if remaining.width <= cfg.width as f32 {
                    result.lines.push(remaining);
                    break;
                }
                let stops = boundaries(&text[start..end]);
                let mut fit = start + stops[1];
                let mut accumulated = 0.0;
                // Logical advance ownership, independent of visual run order.
                for pair in stops.windows(2) {
                    let byte = start + pair[0];
                    accumulated += remaining
                        .glyphs
                        .iter()
                        .filter(|g| g.cluster.start == byte)
                        .map(|g| g.advance)
                        .sum::<f32>();
                    if accumulated > cfg.width as f32 {
                        break;
                    }
                    fit = start + pair[1];
                }
                let soft = unicode_linebreak::linebreaks(&text[start..end])
                    .map(|(i, _)| start + i)
                    .filter(|i| *i <= fit && *i > start && is_boundary(text, *i))
                    .last();
                let mut cut = soft.unwrap_or(fit);
                let line = loop {
                    let line = self.shape_line(
                        text,
                        &bidi,
                        para,
                        start..cut,
                        cfg,
                        &mut result.shape_work,
                    )?;
                    if line.width <= cfg.width as f32 || cut == start + stops[1] {
                        break line;
                    }
                    cut = start
                        + *stops
                            .iter()
                            .rev()
                            .find(|i| start + **i < cut && **i > 0)
                            .ok_or(Error::Bounds)?;
                };
                result.lines.push(line);
                start = cut;
            }
        }
        if text.is_empty() || text.ends_with('\n') {
            result.lines.push(Line {
                bytes: text.len()..text.len(),
                glyphs: Vec::new(),
                carets: vec![Caret {
                    byte: text.len(),
                    x: 0.0,
                }],
                width: 0.0,
            });
        }
        if result.lines.len() > MAX_LINES
            || result.lines.iter().map(|l| l.glyphs.len()).sum::<usize>() > MAX_GLYPHS
        {
            return Err(Error::Bounds);
        }
        Ok(result)
    }
    pub fn rasterize(&mut self, font: usize, glyph: u16, cfg: Config) -> Result<&Bitmap> {
        cfg.validate()?;
        if font >= self.faces.len() || glyph >= self.faces[font].number_of_glyphs() {
            return Err(Error::Bounds);
        }
        let key = RasterKey {
            font,
            glyph,
            size: cfg.size,
            scale: cfg.scale,
        };
        if let Some(i) = self.cache.iter().position(|(k, _)| *k == key) {
            let item = self.cache.remove(i).unwrap();
            self.cache.push_back(item);
        } else {
            // Preflight scaled outline bounds before the rasterizer allocates its bitmap.
            if let Some(b) =
                self.faces[font].glyph_bounding_box(rustybuzz::ttf_parser::GlyphId(glyph))
            {
                let factor = cfg.pixels() / self.faces[font].units_per_em() as f32;
                let width = (b.x_max as f32 - b.x_min as f32) * factor + 2.0;
                let height = (b.y_max as f32 - b.y_min as f32) * factor + 2.0;
                if width > 256.0 || height > 256.0 || width * height > MAX_BITMAP as f32 {
                    return Err(Error::Bounds);
                }
            }
            if self.raster[font].is_none() {
                self.raster[font] = Some(
                    fontdue::Font::from_bytes(
                        self.bytes[font],
                        fontdue::FontSettings {
                            scale: 96.0,
                            ..Default::default()
                        },
                    )
                    .map_err(|_| Error::InvalidFont)?,
                );
            }
            let (m, pixels) = self.raster[font]
                .as_ref()
                .unwrap()
                .rasterize_indexed(glyph, cfg.pixels());
            if pixels.len() > MAX_BITMAP {
                return Err(Error::Bounds);
            }
            while self.cache.len() >= MAX_CACHE_ENTRIES
                || self.cache_bytes + pixels.len() > MAX_CACHE_BYTES
            {
                let (_, old) = self.cache.pop_front().ok_or(Error::Bounds)?;
                self.cache_bytes -= old.pixels.len();
            }
            self.cache_bytes += pixels.len();
            self.cache.push_back((
                key,
                Bitmap {
                    width: m.width,
                    height: m.height,
                    xmin: m.xmin,
                    ymin: m.ymin,
                    pixels,
                },
            ));
        }
        Ok(&self.cache.back().unwrap().1)
    }
}

#[cfg(test)]
mod tests;
