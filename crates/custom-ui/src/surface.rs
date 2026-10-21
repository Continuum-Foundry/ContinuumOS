use crate::*;
use alloc::vec;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lease {
    pub buffer: usize,
    pub generation: u64,
    pub serial: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Released,
    Acquired,
    InUse,
}
struct Buffer {
    bytes: Vec<u8>,
    state: State,
    serial: u64,
    generation: u64,
}
/// Producer cannot obtain mutable pixels after commit. Consumer release is explicit.
pub struct Surface {
    buffers: Vec<Buffer>,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub allocations: u64,
    pub commits: u64,
    pub releases: u64,
}
impl Surface {
    pub fn new(width: u32, height: u32, stride: u32, count: usize) -> Result<Self> {
        if width == 0
            || height == 0
            || width > 4096
            || height > 4096
            || stride < width * 4
            || !stride.is_multiple_of(4)
            || !(1..=3).contains(&count)
        {
            return Err(Error::Bounds);
        }
        let len = (stride as usize)
            .checked_mul(height as usize)
            .filter(|n| *n <= 64 * 1024 * 1024)
            .ok_or(Error::Bounds)?;
        let buffers = (0..count)
            .map(|_| Buffer {
                bytes: vec![0; len],
                state: State::Released,
                serial: 0,
                generation: 0,
            })
            .collect();
        Ok(Self {
            buffers,
            width,
            height,
            stride,
            allocations: count as u64,
            commits: 0,
            releases: 0,
        })
    }
    pub fn acquire(&mut self, generation: u64) -> Result<Lease> {
        if generation == 0 {
            return Err(Error::Invalid);
        }
        let (id, b) = self
            .buffers
            .iter_mut()
            .enumerate()
            .find(|(_, b)| b.state == State::Released && b.serial < u64::MAX)
            .ok_or(Error::Exhausted)?;
        b.serial += 1;
        b.generation = generation;
        b.state = State::Acquired;
        Ok(Lease {
            buffer: id,
            generation,
            serial: b.serial,
        })
    }
    fn buffer(&mut self, l: Lease, state: State) -> Result<&mut Buffer> {
        let b = self.buffers.get_mut(l.buffer).ok_or(Error::Lease)?;
        if b.state != state || b.serial != l.serial || b.generation != l.generation {
            return Err(Error::Lease);
        }
        Ok(b)
    }
    pub fn write(&mut self, l: Lease) -> Result<&mut [u8]> {
        Ok(&mut self.buffer(l, State::Acquired)?.bytes)
    }
    /// Producer abandons an unpublished frame. Never cancels consumer ownership.
    pub fn abandon(&mut self, l: Lease) -> Result<()> {
        self.buffer(l, State::Acquired)?.state = State::Released;
        Ok(())
    }
    pub fn commit(&mut self, l: Lease, damage: &[[u32; 4]]) -> Result<()> {
        if damage.is_empty()
            || damage.len() > 64
            || damage.iter().any(|r| {
                r[0] > self.width
                    || r[1] > self.height
                    || r[2] > self.width - r[0]
                    || r[3] > self.height - r[1]
            })
        {
            return Err(Error::Bounds);
        }
        self.buffer(l, State::Acquired)?.state = State::InUse;
        self.commits += 1;
        Ok(())
    }
    pub fn read(&self, l: Lease) -> Result<&[u8]> {
        let b = self.buffers.get(l.buffer).ok_or(Error::Lease)?;
        if b.state != State::InUse || b.serial != l.serial || b.generation != l.generation {
            return Err(Error::Lease);
        }
        Ok(&b.bytes)
    }
    pub fn release(&mut self, l: Lease) -> Result<()> {
        self.buffer(l, State::InUse)?.state = State::Released;
        self.releases += 1;
        Ok(())
    }
    pub fn allocated_bytes(&self) -> usize {
        self.buffers.iter().map(|b| b.bytes.capacity()).sum()
    }
}
