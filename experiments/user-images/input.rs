//! Experimental P4-06 bounded input policy. Runs in CPL3; portable tests use the same code.
//! US ASCII keymap, physical Linux input key codes, no IME/compose/repeat synthesis.
use super::surface::{Geometry, Peer};
pub const QUEUE: usize = 32;
pub const MAX: usize = 8;
pub use super::input_wire::*;
#[derive(Clone, Copy)]
pub struct Window {
    pub id: u64,
    pub peer: Peer,
    pub scene: u64,
    pub geometry: Geometry,
    pub width: u32,
    pub height: u32,
}
#[derive(Clone, Copy)]
struct Outbox {
    events: [Event; QUEUE],
    read: usize,
    len: usize,
    cancel: Option<Event>,
}
impl Outbox {
    const EMPTY: Self = Self {
        events: [Event::EMPTY; QUEUE],
        read: 0,
        len: 0,
        cancel: None,
    };
    fn peek(&self) -> Option<Event> {
        self.cancel
            .or_else(|| (self.len > 0).then(|| self.events[self.read]))
    }
    fn pop(&mut self) {
        if self.cancel.take().is_none() && self.len > 0 {
            self.read = (self.read + 1) % QUEUE;
            self.len -= 1;
        }
    }
}
pub struct Broker {
    windows: [Option<Window>; MAX],
    out: [Outbox; MAX],
    keys: [bool; 256],
    held_since: [u64; 256],
    button_since: [u64; 3],
    packet_since: [u64; 2],
    buttons: u8,
    focus: Option<usize>,
    capture: Option<usize>,
    press: [Option<(usize, u64)>; 3],
    x: i32,
    y: i32,
    abs: [i32; 2],
    dirty: bool,
    sequence: u64,
    raw: [u64; 2],
    packets: [[(u16, u16, i32); 8]; 2],
    packet_len: [usize; 2],
    time: u64,
    epoch: u64,
    focus_epoch: u64,
    output: u64,
    width: u32,
    height: u32,
    scale: (u32, u32),
    suppressed: bool,
}
impl Broker {
    pub const fn new(width: u32, height: u32) -> Self {
        Self {
            windows: [None; MAX],
            out: [Outbox::EMPTY; MAX],
            keys: [false; 256],
            held_since: [0; 256],
            button_since: [0; 3],
            packet_since: [0; 2],
            buttons: 0,
            focus: None,
            capture: None,
            press: [None; 3],
            x: 0,
            y: 0,
            abs: [0; 2],
            dirty: false,
            sequence: 0,
            raw: [0; 2],
            packets: [[(0, 0, 0); 8]; 2],
            packet_len: [0; 2],
            time: 0,
            epoch: 1,
            focus_epoch: 1,
            output: 1,
            width,
            height,
            scale: (1, 1),
            suppressed: false,
        }
    }
    pub fn dimensions(&mut self, width: u32, height: u32) -> Result<(), ()> {
        if self.sequence != 0
            || self.windows.iter().any(Option::is_some)
            || width == 0
            || width > 1920
            || height == 0
            || height > 1080
        {
            return Err(());
        }
        self.width = width;
        self.height = height;
        Ok(())
    }
    pub fn set_output(&mut self, generation: u64, n: u32, d: u32) -> Result<(), ()> {
        if generation <= self.output || ![(1, 1), (3, 2), (2, 1)].contains(&(n, d)) {
            return Err(());
        }
        self.cancel(4)?;
        self.output = generation;
        self.scale = (n, d);
        Ok(())
    }
    /// Only the compositor's current snapshot populates this table. Clients cannot publish peers/IDs.
    pub fn windows(&mut self, windows: [Option<Window>; MAX]) -> Result<(), ()> {
        for (i, w) in windows.iter().enumerate() {
            if let Some(w) = w {
                if w.id == 0
                    || w.scene == 0
                    || w.peer.session != 1001
                    || w.width == 0
                    || w.height == 0
                    || windows[..i].iter().flatten().any(|p| p.id == w.id)
                {
                    return Err(());
                }
            }
        }
        let invalid = (0..MAX).any(|i| match (self.windows[i], windows[i]) {
            (Some(a), Some(b)) => {
                a.id != b.id || a.peer != b.peer || a.scene != b.scene || a.geometry != b.geometry
            }
            (Some(_), None) => true,
            _ => false,
        });
        if invalid {
            self.cancel(3)?;
        }
        let old = self.windows;
        self.windows = windows;
        // A removed/replaced surface cannot inherit queued events.
        for i in 0..MAX {
            if self.windows[i].is_none()
                || old[i]
                    .zip(self.windows[i])
                    .is_some_and(|(a, b)| a.peer != b.peer || a.id != b.id)
            {
                self.out[i] = Outbox::EMPTY;
            }
        }
        Ok(())
    }
    fn event(
        &mut self,
        i: usize,
        kind: u16,
        code: u16,
        value: i32,
        text: u32,
    ) -> Result<Event, ()> {
        let w = self.windows[i].ok_or(())?;
        self.sequence = self.sequence.checked_add(1).ok_or(())?;
        let (n, d) = self.scale;
        Ok(Event {
            sequence: self.sequence,
            time: self.time,
            epoch: self.epoch,
            surface: w.id,
            scene: w.scene,
            output: self.output,
            focus: self.focus_epoch,
            kind,
            code,
            value,
            x: ((i64::from(self.x) - i64::from(w.geometry.x)) * i64::from(d) / i64::from(n)) as i32,
            y: ((i64::from(self.y) - i64::from(w.geometry.y)) * i64::from(d) / i64::from(n)) as i32,
            modifiers: self.modifiers(),
            text,
        })
    }
    fn emit(&mut self, i: usize, kind: u16, code: u16, value: i32, text: u32) -> Result<(), ()> {
        if self.out[i].len == QUEUE {
            self.cancel(1)?;
            return Ok(());
        }
        let e = self.event(i, kind, code, value, text)?;
        let o = &mut self.out[i];
        o.events[(o.read + o.len) % QUEUE] = e;
        o.len += 1;
        Ok(())
    }
    /// Cancellation is a separately reserved slot. Overflow can never lose all-keys-up.
    pub fn cancel(&mut self, reason: u16) -> Result<(), ()> {
        self.epoch = self.epoch.checked_add(1).ok_or(())?;
        self.focus_epoch = self.focus_epoch.checked_add(1).ok_or(())?;
        self.keys.fill(false);
        self.held_since.fill(0);
        self.button_since = [0; 3];
        self.packet_since = [0; 2];
        self.buttons = 0;
        self.focus = None;
        self.capture = None;
        self.press = [None; 3];
        self.dirty = false;
        self.packet_len = [0; 2];
        self.suppressed = true;
        for i in 0..MAX {
            if self.windows[i].is_some() {
                let dropped = self.out[i].len + usize::from(self.out[i].cancel.is_some());
                let e = self.event(i, CANCEL, reason, dropped as i32, 0)?;
                self.out[i] = Outbox {
                    cancel: Some(e),
                    ..Outbox::EMPTY
                };
            }
        }
        Ok(())
    }
    pub fn focus(&mut self, caller: Peer, id: u64, scene: u64) -> Result<(), ()> {
        let i = self.authorize(caller, id, scene)?;
        if !self.windows[i].unwrap().geometry.visible {
            return Err(());
        }
        if self.focus != Some(i) {
            self.cancel(2)?;
            self.focus = Some(i);
            self.suppressed = false;
            self.emit(i, FOCUS, 0, 1, 0)?;
        }
        Ok(())
    }
    pub fn capture(&mut self, caller: Peer, id: u64, scene: u64) -> Result<(), ()> {
        let i = self.authorize(caller, id, scene)?;
        if self.focus != Some(i)
            || !self.press.iter().any(|p| *p == Some((i, scene)))
            || self.buttons == 0
        {
            return Err(());
        }
        self.capture = Some(i);
        Ok(())
    }
    fn authorize(&self, p: Peer, id: u64, scene: u64) -> Result<usize, ()> {
        self.windows
            .iter()
            .position(|w| {
                w.is_some_and(|w| {
                    w.id == id && w.peer == p && p.session == 1001 && w.scene == scene
                })
            })
            .ok_or(())
    }
    pub fn peek(&self, p: Peer, id: u64, scene: u64) -> Result<Option<Event>, ()> {
        let i = self.authorize(p, id, scene)?;
        Ok(self.out[i].peek())
    }
    pub fn consumed(&mut self, p: Peer, id: u64, scene: u64) -> Result<(), ()> {
        let i = self.authorize(p, id, scene)?;
        self.out[i].pop();
        Ok(())
    }
    fn hit(&self) -> Option<usize> {
        self.windows
            .iter()
            .enumerate()
            .filter_map(|(i, w)| {
                let w = w.as_ref()?;
                let g = w.geometry;
                let x = i64::from(self.x) - i64::from(g.x);
                let y = i64::from(self.y) - i64::from(g.y);
                let c = g.clip;
                (g.visible
                    && x >= i64::from(c[0])
                    && y >= i64::from(c[1])
                    && x < i64::from(c[0]) + i64::from(c[2])
                    && y < i64::from(c[1]) + i64::from(c[3])
                    && x < i64::from(w.width)
                    && y < i64::from(w.height))
                .then_some((i, g.z, w.id))
            })
            .max_by_key(|&(_, z, id)| (z, id))
            .map(|(i, _, _)| i)
    }
    fn modifiers(&self) -> u32 {
        u32::from(self.keys[42] || self.keys[54])
            | u32::from(self.keys[29] || self.keys[97]) << 1
            | u32::from(self.keys[56] || self.keys[100]) << 2
    }
    /// Trusted raw envelope: independent per-device sequence, monotonic host-independent guest ns.
    /// Device 0 keyboard, 1 absolute tablet; only advertised selected EV codes admitted.
    pub fn raw(
        &mut self,
        device: usize,
        sequence: u64,
        time: u64,
        kind: u16,
        code: u16,
        value: i32,
    ) -> Result<(), ()> {
        if device >= 2 || sequence != self.raw[device].checked_add(1).ok_or(())? || time < self.time
        {
            self.cancel(5)?;
            return Err(());
        }
        self.raw[device] = sequence;
        self.time = time;
        let valid = match (device, kind, code, value) {
            (_, 0, 0, 0) => true,
            (_, 0, 3, _) => {
                self.cancel(6)?;
                return Ok(());
            }
            (0, 1, 1..=127, 0..=2) => true,
            (1, 1, 272..=274, 0..=1) => true,
            (1, 3, 0..=1, 0..=32767) => true,
            _ => false,
        };
        if !valid {
            self.cancel(7)?;
            return Err(());
        }
        if kind == 0 {
            let packet = self.packets[device];
            let len = self.packet_len[device];
            self.packet_len[device] = 0;
            for &(k, c, v) in &packet[..len] {
                self.apply(device, k, c, v)?;
            }
            return self.apply(device, 0, 0, 0);
        }
        let n = self.packet_len[device];
        if n == 8 {
            self.cancel(1)?;
            return Err(());
        }
        if n == 0 {
            self.packet_since[device] = self.time;
        }
        self.packets[device][n] = (kind, code, value);
        self.packet_len[device] += 1;
        Ok(())
    }
    fn apply(&mut self, device: usize, kind: u16, code: u16, mut value: i32) -> Result<(), ()> {
        if kind == 0 {
            if self.dirty {
                self.x = (i64::from(self.abs[0]) * i64::from(self.width.saturating_sub(1)) / 32767)
                    as i32;
                self.y = (i64::from(self.abs[1]) * i64::from(self.height.saturating_sub(1)) / 32767)
                    as i32;
                self.dirty = false;
                if let Some(i) = self.capture.or_else(|| self.hit()) {
                    self.emit(i, MOTION, 0, 0, 0)?;
                }
            }
            return Ok(());
        }
        if kind == 3 {
            self.abs[code as usize] = value;
            self.dirty = true;
            return Ok(());
        }
        if device == 0 {
            let was = self.keys[code as usize];
            if value == 2 && !was || value == 0 && !was {
                return Ok(());
            } // orphan release/repeat after cancellation
            if value == 1 && was {
                value = 2;
            } // QEMU HID encodes host auto-repeat as another down.
            self.keys[code as usize] = value != 0;
            self.held_since[code as usize] = if value != 0 { self.time } else { 0 };
            if let Some(i) = self.focus {
                self.emit(i, KEY, code, value, 0)?;
                if value != 0 {
                    let ch = ascii(code, self.modifiers());
                    if ch != 0 && !self.suppressed {
                        self.emit(i, TEXT, code, value, ch)?;
                    }
                }
            }
            return Ok(());
        }
        // Flush ABS coordinates before buttons even if a device emits its SYN afterwards.
        if self.dirty {
            self.x =
                (i64::from(self.abs[0]) * i64::from(self.width.saturating_sub(1)) / 32767) as i32;
            self.y =
                (i64::from(self.abs[1]) * i64::from(self.height.saturating_sub(1)) / 32767) as i32;
            self.dirty = false;
            if let Some(i) = self.capture.or_else(|| self.hit()) {
                self.emit(i, MOTION, 0, 0, 0)?;
            }
        }
        let bit = 1 << (code - 272);
        if value == 1 {
            if self.buttons & bit != 0 {
                return Ok(());
            }
            if let Some(i) = self.capture.or_else(|| self.hit()) {
                let w = self.windows[i].unwrap();
                if self.focus != Some(i) {
                    self.focus(w.peer, w.id, w.scene)?;
                }
                self.buttons |= bit;
                self.button_since[(code - 272) as usize] = self.time;
                self.press[(code - 272) as usize] = Some((i, w.scene));
                self.emit(i, BUTTON, code, 1, 0)?;
            }
        } else if self.buttons & bit != 0 {
            self.buttons &= !bit;
            self.button_since[(code - 272) as usize] = 0;
            if let Some((i, scene)) = self.press[(code - 272) as usize].take() {
                if self.windows[i].is_some_and(|w| w.scene == scene)
                    && (self.capture == Some(i) || self.hit() == Some(i))
                {
                    self.emit(i, BUTTON, code, 0, 0)?;
                } else {
                    self.cancel(8)?;
                }
            }
            if self.buttons == 0 {
                self.capture = None;
                self.press = [None; 3];
            }
        }
        Ok(())
    }
    pub fn device_sequence(&self, device: usize) -> u64 {
        self.raw[device]
    }
    pub fn idle(&mut self, now: u64) -> Result<(), ()> {
        let expired = self
            .keys
            .iter()
            .zip(self.held_since)
            .any(|(&held, t)| held && now.saturating_sub(t) > 2_000_000_000)
            || (0..3).any(|i| {
                self.buttons & (1 << i) != 0
                    && now.saturating_sub(self.button_since[i]) > 2_000_000_000
            })
            || (0..2).any(|i| {
                self.packet_len[i] > 0 && now.saturating_sub(self.packet_since[i]) > 2_000_000_000
            });
        if expired {
            self.time = now;
            self.cancel(9)?;
        }
        Ok(())
    }
    pub fn reset(&mut self) -> Result<(), ()> {
        self.raw = [0; 2];
        self.cancel(6)
    }
}
fn ascii(code: u16, modifiers: u32) -> u32 {
    if modifiers & 6 != 0 {
        return 0;
    }
    let shift = modifiers & 1 != 0;
    let (plain, upper): (&[u8], &[u8]) = (b"1234567890-=", b"!@#$%^&*()_+");
    if (2..=13).contains(&code) {
        return if shift {
            upper[(code - 2) as usize]
        } else {
            plain[(code - 2) as usize]
        } as u32;
    }
    for (start, row) in [
        (16, b"qwertyuiop" as &[u8]),
        (30, b"asdfghjkl"),
        (44, b"zxcvbnm"),
    ] {
        if code >= start && (code - start) < row.len() as u16 {
            let c = row[(code - start) as usize];
            return if shift { c.to_ascii_uppercase() } else { c } as u32;
        }
    }
    let pair = match code {
        26 => [b'[', b'{'],
        27 => [b']', b'}'],
        39 => [b';', b':'],
        40 => [b'\'', b'"'],
        41 => [b'`', b'~'],
        43 => [b'\\', b'|'],
        51 => [b',', b'<'],
        52 => [b'.', b'>'],
        53 => [b'/', b'?'],
        57 => [b' ', b' '],
        28 => [b'\n', b'\n'],
        15 => [b'\t', b'\t'],
        _ => return 0,
    };
    pair[usize::from(shift)] as u32
}
#[cfg(test)]
mod tests {
    use super::*;
    const P: Peer = Peer {
        process: 4,
        incarnation: 1,
        principal: 403,
        session: 1001,
    };
    fn window() -> Window {
        Window {
            id: 1,
            peer: P,
            scene: 1,
            geometry: Geometry {
                x: 10,
                y: 10,
                z: 0,
                visible: true,
                clip: [0, 0, 100, 100],
            },
            width: 100,
            height: 100,
        }
    }
    fn broker() -> Broker {
        let mut b = Broker::new(1280, 720);
        let mut w = [None; MAX];
        w[0] = Some(window());
        b.windows(w).unwrap();
        b.focus(P, 1, 1).unwrap();
        while b.peek(P, 1, 1).unwrap().is_some() {
            b.consumed(P, 1, 1).unwrap();
        }
        b
    }
    fn policy(
        b: &mut Broker,
        device: usize,
        sequence: u64,
        time: u64,
        kind: u16,
        code: u16,
        value: i32,
    ) -> Result<(), ()> {
        // Deliberately bypass the device parser to isolate routing/key-state policy.
        if device >= 2 || sequence != b.raw[device] + 1 || time < b.time {
            b.cancel(5)?;
            return Err(());
        }
        b.raw[device] = sequence;
        b.time = time;
        match (device, kind, code, value) {
            (_, 0, 0, 0)
            | (0, 1, 1..=127, 0..=2)
            | (1, 1, 272..=274, 0..=1)
            | (1, 3, 0..=1, 0..=32767) => b.apply(device, kind, code, value),
            _ => {
                b.cancel(7)?;
                Err(())
            }
        }
    }
    #[test]
    fn key_modifier_repeat_and_literal_wire() {
        let mut b = broker();
        policy(&mut b, 0, 1, 10, 1, 42, 1).unwrap();
        policy(&mut b, 0, 2, 11, 1, 30, 1).unwrap();
        policy(&mut b, 0, 3, 12, 1, 30, 2).unwrap();
        policy(&mut b, 0, 4, 13, 1, 30, 0).unwrap();
        policy(&mut b, 0, 5, 14, 1, 42, 0).unwrap();
        let mut events = [Event::EMPTY; 8];
        let mut n = 0;
        while let Some(e) = b.peek(P, 1, 1).unwrap() {
            events[n] = e;
            n += 1;
            b.consumed(P, 1, 1).unwrap();
        }
        assert_eq!(n, 7);
        assert_eq!((events[2].kind, events[2].text), (TEXT, 65));
        assert_eq!(events[3].value, 2);
        assert_eq!(events[6].modifiers, 0);
        let bytes = events[2].encode();
        assert_eq!(&bytes[..8], b"CIN0\0\0\x60\0");
        assert_eq!(&bytes[64..68], &[2, 0, 30, 0]);
        assert_eq!(Event::decode(&bytes).unwrap(), events[2]);
        let mut bad = bytes;
        bad[95] = 1;
        assert!(Event::decode(&bad).is_err());
    }
    #[test]
    fn overflow_cancel_is_reserved_and_orphan_release_ignored() {
        let mut b = broker();
        for n in 1..40 {
            policy(&mut b, 0, n, n, 1, 42, if n % 2 == 1 { 1 } else { 0 }).unwrap();
        }
        let e = b.peek(P, 1, 1).unwrap().unwrap();
        assert_eq!((e.kind, e.code), (CANCEL, 1));
        assert!(b.focus.is_none());
        assert!(b.capture.is_none());
        assert_eq!(b.buttons, 0);
    }
    #[test]
    fn authority_stale_geometry_capture_and_missing_release() {
        let mut b = broker();
        assert!(b.focus(Peer { session: 2002, ..P }, 1, 1).is_err());
        assert!(b.capture(P, 1, 1).is_err());
        policy(&mut b, 1, 1, 1, 3, 0, 800).unwrap();
        policy(&mut b, 1, 2, 2, 3, 1, 1000).unwrap();
        policy(&mut b, 1, 3, 3, 1, 272, 1).unwrap();
        b.capture(P, 1, 1).unwrap();
        let mut w = window();
        w.scene = 2;
        w.geometry.x = 200;
        let mut ws = [None; MAX];
        ws[0] = Some(w);
        b.windows(ws).unwrap();
        assert!(b.capture.is_none());
        assert_eq!(b.peek(P, 1, 2).unwrap().unwrap().kind, CANCEL);
        policy(&mut b, 1, 4, 4, 1, 272, 0).unwrap();
        assert_eq!(b.out[0].len, 0);
        b.focus(P, 1, 2).unwrap();
        policy(&mut b, 0, 1, 5, 1, 30, 1).unwrap();
        b.idle(2_000_000_006).unwrap();
        assert!(!b.keys.iter().any(|k| *k));
    }
    #[test]
    fn malformed_sequence_reset_and_fractional_coordinates() {
        let mut b = broker();
        assert!(policy(&mut b, 0, 2, 2, 1, 30, 1).is_err());
        b.set_output(2, 3, 2).unwrap();
        b.focus(P, 1, 1).unwrap();
        policy(&mut b, 1, 1, 3, 3, 0, 1000).unwrap();
        policy(&mut b, 1, 2, 4, 3, 1, 1000).unwrap();
        policy(&mut b, 1, 3, 5, 0, 0, 0).unwrap();
        let e = b.out[0].events[(b.out[0].read + b.out[0].len - 1) % QUEUE];
        assert_eq!((e.x, e.y), (19, 7));
        assert!(policy(&mut b, 1, 4, 6, 3, 0, 32768).is_err());
    }
    #[test]
    fn device_reports_commit_only_at_syn_and_reject_incomplete_or_malformed_packets() {
        let mut b = broker();
        b.raw(0, 1, 1, 1, 30, 1).unwrap();
        assert!(b.peek(P, 1, 1).unwrap().is_none());
        b.raw(0, 2, 2, 0, 0, 0).unwrap();
        let e = b.peek(P, 1, 1).unwrap().unwrap();
        assert_eq!((e.kind, e.code, e.value), (KEY, 30, 1));
        b.cancel(6).unwrap();
        b.focus(P, 1, 1).unwrap();
        while b.peek(P, 1, 1).unwrap().is_some() {
            b.consumed(P, 1, 1).unwrap();
        }
        b.raw(0, 3, 3, 1, 31, 1).unwrap();
        assert!(b.raw(0, 4, 4, 77, 0, 0).is_err());
        assert_eq!(b.peek(P, 1, 1).unwrap().unwrap().kind, CANCEL);
        assert!(!b.keys.iter().any(|k| *k));
        b.raw(0, 5, 5, 1, 32, 1).unwrap();
        b.idle(2_000_000_006).unwrap();
        assert_eq!(b.packet_len, [0, 0]);
    }
    #[test]
    fn owner_replacement_death_and_hide_cannot_inherit_capture_or_queued_keys() {
        let mut b = broker();
        policy(&mut b, 0, 1, 1, 1, 30, 1).unwrap();
        let mut w = window();
        w.peer.incarnation = 2;
        let mut ws = [None; MAX];
        ws[0] = Some(w);
        b.windows(ws).unwrap();
        assert!(b.peek(w.peer, 1, 1).unwrap().is_none());
        assert!(b.focus.is_none());
        b.focus(w.peer, 1, 1).unwrap();
        w.geometry.visible = false;
        ws[0] = Some(w);
        b.windows(ws).unwrap();
        assert!(b.focus(w.peer, 1, 1).is_err());
        assert_eq!(b.peek(w.peer, 1, 1).unwrap().unwrap().kind, CANCEL);
        b.windows([None; MAX]).unwrap();
        assert!(b.focus.is_none() && b.capture.is_none());
        assert!(b.peek(w.peer, 1, 1).is_err());
    }
    #[test]
    fn unrelated_pointer_activity_cannot_keep_a_missing_keyboard_release_or_partial_packet_alive() {
        let mut b = broker();
        b.raw(0, 1, 1, 1, 42, 1).unwrap();
        b.raw(0, 2, 2, 0, 0, 0).unwrap();
        b.raw(1, 1, 1_999_999_999, 3, 0, 500).unwrap();
        b.raw(1, 2, 2_000_000_000, 0, 0, 0).unwrap();
        b.idle(2_000_000_003).unwrap();
        assert!(!b.keys[42]);
        assert_eq!(b.peek(P, 1, 1).unwrap().unwrap().code, 9);
        b.raw(0, 3, 3_000_000_000, 1, 30, 1).unwrap();
        b.raw(1, 3, 4_999_999_999, 3, 0, 501).unwrap();
        b.raw(1, 4, 5_000_000_000, 0, 0, 0).unwrap();
        b.idle(5_000_000_001).unwrap();
        assert_eq!(b.packet_len, [0, 0]);
    }
    #[test]
    fn capture_routes_secondary_buttons_outside_window_and_releases_each_press_once() {
        let mut b = broker();
        let mut other = window();
        other.id = 2;
        other.peer = Peer {
            process: 5,
            principal: 404,
            ..P
        };
        other.geometry.x = 200;
        let mut ws = [None; MAX];
        ws[0] = Some(window());
        ws[1] = Some(other);
        b.windows(ws).unwrap();
        policy(&mut b, 1, 1, 1, 3, 0, 800).unwrap();
        policy(&mut b, 1, 2, 2, 3, 1, 1000).unwrap();
        policy(&mut b, 1, 3, 3, 1, 272, 1).unwrap();
        b.capture(P, 1, 1).unwrap();
        policy(&mut b, 1, 4, 4, 3, 0, 6000).unwrap();
        policy(&mut b, 1, 5, 5, 1, 273, 1).unwrap();
        policy(&mut b, 1, 6, 6, 1, 272, 0).unwrap();
        assert_eq!(b.capture, Some(0));
        policy(&mut b, 1, 7, 7, 1, 273, 0).unwrap();
        assert!(b.capture.is_none());
        let mut buttons = [(0, 0); 4];
        let mut count = 0;
        while let Some(e) = b.peek(P, 1, 1).unwrap() {
            if e.kind == BUTTON {
                buttons[count] = (e.code, e.value);
                count += 1;
            }
            b.consumed(P, 1, 1).unwrap();
        }
        assert_eq!(count, 4);
        assert_eq!(buttons, [(272, 1), (273, 1), (272, 0), (273, 0)]);
        assert_eq!(b.buttons, 0);
        assert!(b.peek(other.peer, 2, 1).unwrap().is_none());
    }
}
