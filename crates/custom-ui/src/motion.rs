//! Explicit monotonic deadlines. No worker, wall clock or unbounded catch-up loop.
use crate::*;

#[derive(Clone, Debug)]
struct Transition {
    id: String,
    from: f32,
    to: f32,
    start: u64,
    duration: u64,
}
impl Transition {
    fn value(&self, now: u64) -> f32 {
        if self.duration == 0 {
            return self.to;
        }
        let t = (now.saturating_sub(self.start) as f32 / self.duration.max(1) as f32).min(1.0);
        self.from + (self.to - self.from) * t
    }
}
#[derive(Default)]
pub struct Motion {
    transitions: Vec<Transition>,
    now: u64,
    pub caret_visible: bool,
    caret: Option<u64>,
    clock: Option<u64>,
    drag: Option<u64>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tick {
    pub paint: bool,
    pub clock: bool,
    pub drag: bool,
}
impl Motion {
    pub fn opacity(&self, id: &str, base: f32) -> f32 {
        self.transitions
            .iter()
            .find(|t| t.id == id)
            .map(|t| t.value(self.now))
            .unwrap_or(base)
    }
    pub fn transition(
        &mut self,
        id: &str,
        base: f32,
        to: f32,
        now: u64,
        reduced: bool,
    ) -> Result<()> {
        if !valid_id(id)
            || [base, to]
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
            || now < self.now
        {
            return Err(Error::Bounds);
        }
        let from = self
            .transitions
            .iter()
            .find(|t| t.id == id)
            .map(|t| t.value(now))
            .unwrap_or(base);
        if !self.transitions.iter().any(|t| t.id == id) && self.transitions.len() == 64 {
            return Err(Error::Exhausted);
        }
        self.now = now;
        self.transitions.retain(|t| t.id != id);
        self.transitions.push(Transition {
            id: id.into(),
            from,
            to,
            start: now,
            duration: if reduced { 0 } else { 120_000 },
        });
        Ok(())
    }
    pub fn timers(
        &mut self,
        caret: bool,
        clock: bool,
        drag: bool,
        now: u64,
        reduced: bool,
    ) -> Result<()> {
        if now < self.now {
            return Err(Error::Input);
        }
        self.now = now;
        self.caret_visible = caret;
        self.caret = if caret && !reduced {
            Some(now.checked_add(500_000).ok_or(Error::Bounds)?)
        } else {
            None
        };
        self.clock = if clock {
            Some(now.checked_add(1_000_000).ok_or(Error::Bounds)?)
        } else {
            None
        };
        self.drag = if drag {
            Some(now.checked_add(16_667).ok_or(Error::Bounds)?)
        } else {
            None
        };
        Ok(())
    }
    pub fn next_deadline(&self) -> Option<u64> {
        self.transitions
            .iter()
            .filter(|t| self.now < t.start.saturating_add(t.duration))
            .map(|t| (self.now.saturating_add(16_667)).min(t.start.saturating_add(t.duration)))
            .chain(self.caret)
            .chain(self.clock)
            .chain(self.drag)
            .min()
    }
    pub fn tick(&mut self, now: u64) -> Result<Tick> {
        if now < self.now {
            return Err(Error::Input);
        }
        let mut out = Tick::default();
        out.paint = self
            .transitions
            .iter()
            .any(|t| self.now < t.start.saturating_add(t.duration) && now > self.now);
        self.now = now;
        for (deadline, period, which) in [
            (&mut self.caret, 500_000, 0),
            (&mut self.clock, 1_000_000, 1),
            (&mut self.drag, 16_667, 2),
        ] {
            if deadline.is_some_and(|d| now >= d) {
                // Skip elapsed periods in O(1); caret phase stays deterministic.
                let old = deadline.unwrap();
                let periods = (now - old) / period + 1;
                *deadline = old.checked_add(periods.saturating_mul(period));
                match which {
                    0 => {
                        if periods % 2 == 1 {
                            self.caret_visible = !self.caret_visible;
                        }
                        out.paint = true;
                    }
                    1 => out.clock = true,
                    _ => out.drag = true,
                }
            }
        }
        Ok(out)
    }
    pub fn settle(&mut self) {
        for t in &mut self.transitions {
            t.from = t.to;
            t.duration = 0;
        }
        self.caret = None;
        self.caret_visible = true;
    }
    pub fn retain(&mut self, tree: &crate::scene::Tree) {
        self.transitions.retain(|t| tree.node(&t.id).is_ok());
    }
    pub fn count(&self) -> usize {
        self.transitions.len()
    }
}
