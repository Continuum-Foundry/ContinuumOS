//! Private desktop copy transport. No retained user pointers or caller-supplied identity.
//! One request/response per client bounds storage and provides backpressure.
pub const BYTES: usize = 4096;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Peer {
    pub process: u64,
    pub incarnation: u64,
    pub principal: u64,
    pub session: u64,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Empty,
    Queued,
    Reading,
    Reply,
}
struct Slot {
    state: State,
    peer: Option<Peer>,
    bytes: [u8; BYTES],
    len: usize,
    ticket: u64,
}
impl Slot {
    const EMPTY: Self = Self {
        state: State::Empty,
        peer: None,
        bytes: [0; BYTES],
        len: 0,
        ticket: 0,
    };
    fn clear(&mut self) {
        self.state = State::Empty;
        self.peer = None;
        self.bytes.fill(0);
        self.len = 0;
    }
}
pub struct Mailbox {
    slots: [Slot; 2],
    next: u64,
    cursor: usize,
}
impl Mailbox {
    pub const EMPTY: Self = Self {
        slots: [const { Slot::EMPTY }; 2],
        next: 0,
        cursor: 0,
    };
    pub fn submit(&mut self, client: usize, peer: Peer, bytes: &[u8]) -> Result<u64, ()> {
        let s = self.slots.get_mut(client).ok_or(())?;
        if s.state != State::Empty || bytes.is_empty() || bytes.len() > BYTES {
            return Err(());
        }
        let next = self.next.checked_add(1).ok_or(())?;
        s.bytes[..bytes.len()].copy_from_slice(bytes);
        s.len = bytes.len();
        s.peer = Some(peer);
        s.ticket = next;
        s.state = State::Queued;
        self.next = next;
        Ok(next)
    }
    /// Peek never consumes on a failed destination copy. Caller marks reading afterwards.
    pub fn peek(&self) -> Option<(usize, Peer, u64, &[u8])> {
        (0..2).map(|n| (self.cursor + n) % 2).find_map(|i| {
            let s = &self.slots[i];
            (s.state == State::Queued).then(|| (i, s.peer.unwrap(), s.ticket, &s.bytes[..s.len]))
        })
    }
    pub fn reading(&mut self, client: usize) {
        self.slots[client].state = State::Reading;
        self.cursor = (client + 1) % 2;
    }
    pub fn reply(&mut self, ticket: u64, bytes: &[u8]) -> Result<(), ()> {
        let s = self
            .slots
            .iter_mut()
            .find(|s| s.ticket == ticket && s.state == State::Reading)
            .ok_or(())?;
        if bytes.len() > BYTES {
            return Err(());
        }
        s.bytes.fill(0);
        s.bytes[..bytes.len()].copy_from_slice(bytes);
        s.len = bytes.len();
        s.state = State::Reply;
        Ok(())
    }
    pub fn response(&self, client: usize, peer: Peer, ticket: u64) -> Result<Option<&[u8]>, ()> {
        let s = self.slots.get(client).ok_or(())?;
        if s.peer != Some(peer) || s.ticket != ticket || s.state == State::Empty {
            return Err(());
        }
        Ok((s.state == State::Reply).then_some(&s.bytes[..s.len]))
    }
    pub fn consumed(&mut self, client: usize) {
        self.slots[client].clear();
    }
    pub fn dead(&mut self, peer: Peer) {
        for s in &mut self.slots {
            if s.peer == Some(peer) {
                s.clear();
            }
        }
    }
    /// Compositor death cancels incomplete requests; completed copied replies survive until consumed.
    pub fn reset(&mut self) {
        for s in &mut self.slots {
            if s.state != State::Reply {
                s.clear();
            }
        }
    }
    pub fn live(&self) -> usize {
        self.slots
            .iter()
            .filter(|s| s.state != State::Empty)
            .count()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    const A: Peer = Peer {
        process: 4,
        incarnation: 1,
        principal: 403,
        session: 1001,
    };
    #[test]
    fn pressure_copy_failure_death_and_stale_reply() {
        let mut m = Mailbox::EMPTY;
        let mut data = [7; BYTES];
        let t = m.submit(0, A, &data).unwrap();
        data.fill(8);
        assert!(m.submit(0, A, &data).is_err());
        assert!(m.submit(2, A, &data).is_err());
        assert_eq!(m.peek().unwrap().3, [7; BYTES]);
        assert_eq!(m.peek().unwrap().2, t);
        m.reading(0);
        assert!(m.peek().is_none());
        assert!(m.reply(t + 1, &[1]).is_err());
        m.reply(t, &[9]).unwrap();
        assert!(m.response(0, Peer { session: 2, ..A }, t).is_err());
        assert_eq!(m.response(0, A, t).unwrap(), Some(&[9][..]));
        m.consumed(0);
        assert!(m.response(0, A, t).is_err());
        let t2 = m.submit(0, A, &data).unwrap();
        assert!(t2 > t);
        m.reading(0);
        m.dead(A);
        assert_eq!(m.live(), 0);
        assert!(m.reply(t2, &[1]).is_err());
        m.submit(1, A, &data).unwrap();
        m.reset();
        assert_eq!(m.live(), 0);
    }
}
