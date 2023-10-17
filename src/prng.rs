use anyhow::{bail, Result};
use std::num::Wrapping;
use std::ops::Range;

pub trait Prng {
    fn from_seed(seed: i32) -> Result<Self>
    where
        Self: Sized;
    fn gen_range(&mut self, range: Range<i32>) -> Result<i32>;
    fn gen_float(&mut self) -> Result<f64>;
}

pub struct Jkiss {
    x: Wrapping<u32>,
    y: Wrapping<u32>,
    z: Wrapping<u32>,
    c: Wrapping<u32>,
}

impl Jkiss {
    fn gen(&mut self) -> u32 {
        self.x = Wrapping(314527869u32) * self.x + Wrapping(1234567u32);

        self.y ^= self.y << 5usize;
        self.y ^= self.y >> 7usize;
        self.y ^= self.y << 22usize;

        // This will never overflow. The maximum value is 0xfffa28490005d7b6.
        let t: u64 = 4294584393u64 * (self.z.0 as u64) + (self.c.0 as u64);
        self.z = Wrapping(t as u32);
        self.c = Wrapping((t >> 32usize) as u32);

        (self.x + self.y + self.z).0
    }
}

// Very similar to:
// https://github.com/mono/mono/commit/8d3c6d44f8388897fd4d53e819637bf5ee82cfed#diff-69d4dc59d30a768318a79c254ef6d6041cc591deb6439355402317e23b1da5ad
// The only difference seems to be how x is initially assigned.
impl Prng for Jkiss {
    fn from_seed(seed: i32) -> Result<Self> {
        Ok(Jkiss {
            x: Wrapping(314527869u32) * Wrapping(seed as u32) + Wrapping(1234567u32),
            y: Wrapping(987654321u32),
            z: Wrapping(43219876u32),
            c: Wrapping(6543217u32),
        })
    }

    fn gen_range(&mut self, range: Range<i32>) -> Result<i32> {
        if range.is_empty() {
            bail!("Empty range parsed to Jkiss.gen_range().");
        }

        let difference: u32 = (range.end - range.start) as u32;
        if difference == 1u32 {
            Ok(range.start)
        } else {
            Ok(range.start + (self.gen() % difference) as i32)
        }
    }

    fn gen_float(&mut self) -> Result<f64> {
        let a: f64 = (self.gen() >> 6) as f64;
        let b: f64 = (self.gen() >> 6) as f64;

        Ok((a * 134217728f64 + b) / 9007199254740992f64)
    }
}
