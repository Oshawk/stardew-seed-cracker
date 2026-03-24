use std::num::Wrapping;
use std::ops::Range;

use crate::traveling_merchant::Platform;

pub trait Prng {
    fn from_seed(seed: i32) -> Self
    where
        Self: Sized;

    /// C#'s `Next()` — returns a non-negative i32.
    fn next(&mut self) -> i32;

    /// C#'s `Next(maxValue)` — returns a value in [0, max).
    fn next_max(&mut self, max: i32) -> i32;

    /// C#'s `Next(minValue, maxValue)` — returns a value in [min, max).
    fn next_min_max(&mut self, range: Range<i32>) -> i32;

    /// C#'s `NextDouble()` — returns a double in [0, 1).
    fn next_double(&mut self) -> f64;
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

// Identical to:
// https://github.com/mono/mono/commit/8d3c6d44f8388897fd4d53e819637bf5ee82cfed#diff-69d4dc59d30a768318a79c254ef6d6041cc591deb6439355402317e23b1da5ad
impl Prng for Jkiss {
    fn from_seed(seed: i32) -> Self {
        Jkiss {
            x: Wrapping(seed as u32),
            y: Wrapping(987654321u32),
            z: Wrapping(43219876u32),
            c: Wrapping(6543217u32),
        }
    }

    fn next(&mut self) -> i32 {
        loop {
            let random = self.gen() as i32;
            if random == i32::MIN {
                continue;
            }
            let mask = random >> 31;
            return (random ^ mask) + (mask & 1);
        }
    }

    fn next_max(&mut self, max: i32) -> i32 {
        if max <= 0 {
            return 0;
        }
        (self.gen() % max as u32) as i32
    }

    fn next_min_max(&mut self, range: Range<i32>) -> i32 {
        let difference: u32 = (Wrapping(range.end) - Wrapping(range.start)).0 as u32;
        if difference <= 1 {
            range.start
        } else {
            (Wrapping(range.start) + Wrapping((self.gen() % difference) as i32)).0
        }
    }

    fn next_double(&mut self) -> f64 {
        let a: f64 = (self.gen() >> 6) as f64;
        let b: f64 = (self.gen() >> 5) as f64;

        (a * 134217728f64 + b) / 9007199254740992f64
    }
}

// https://github.com/microsoft/referencesource/blob/master/mscorlib/system/random.cs
// The generator doesn't seem to overflow, so we don't need to use Wrapping much.
pub struct MsCorLibRandom {
    seed: [i32; 56usize],
    n: usize,
    np: usize,
}

impl MsCorLibRandom {
    fn gen(&mut self) -> i32 {
        self.n += 1usize;
        if self.n >= 56usize {
            self.n = 1usize;
        }

        self.np += 1usize;
        if self.np >= 56usize {
            self.np = 1usize;
        }

        let mut result: i32 = self.seed[self.n] - self.seed[self.np];

        if result == i32::MAX {
            result -= 1i32;
        }
        if result < 0i32 {
            result += i32::MAX;
        }

        self.seed[self.n] = result;

        result
    }
}

impl Prng for MsCorLibRandom {
    fn from_seed(seed: i32) -> Self {
        let mut s: Self = Self {
            seed: [0i32; 56usize],
            n: 0usize,
            np: 21usize,
        };

        let subtraction: i32 = match seed {
            i32::MIN => i32::MAX,
            _ => seed.abs(),
        };

        let mut mj: i32 = 161803398i32 - subtraction;
        s.seed[55usize] = mj;

        let mut mk: i32 = 1i32;
        for i in 1usize..55usize {
            let ii: usize = (21usize * i) % 55usize;
            s.seed[ii] = mk;
            mk = mj.wrapping_sub(mk);
            if mk < 0i32 {
                mk += i32::MAX;
            }
            mj = s.seed[ii];
        }

        for _ in 1usize..5usize {
            for i in 1usize..56usize {
                s.seed[i] = s.seed[i].wrapping_sub(s.seed[1usize + (i + 30usize) % 55usize]);
                if s.seed[i] == i32::MAX {
                    s.seed[i] -= 1i32;
                }
                if s.seed[i] < 0i32 {
                    s.seed[i] += i32::MAX;
                }
            }
        }

        s
    }

    fn next(&mut self) -> i32 {
        self.gen()
    }

    fn next_max(&mut self, max: i32) -> i32 {
        (self.next_double() * max as f64) as i32
    }

    fn next_min_max(&mut self, range: Range<i32>) -> i32 {
        let difference: u32 = (Wrapping(range.end) - Wrapping(range.start)).0 as u32;
        if difference <= i32::MAX as u32 {
            range.start + (self.next_double() * difference as f64) as i32
        } else {
            let mut sample: i32 = self.gen();
            if self.gen() % 2i32 == 0i32 {
                sample = -sample;
            }

            let mut sample: f64 = sample as f64;
            sample += (i32::MAX - 1i32) as f64;
            sample /= ((2u32 * i32::MAX as u32) - 1u32) as f64;

            (Wrapping(range.start) + Wrapping((sample * difference as f64) as u32 as i32)).0
        }
    }

    fn next_double(&mut self) -> f64 {
        self.gen() as f64 * (1f64 / i32::MAX as f64)
    }
}

pub fn get_prng(platform: Platform, seed: i32) -> Box<dyn Prng> {
    match platform {
        Platform::Switch => Box::new(Jkiss::from_seed(seed)),
        Platform::PC => Box::new(MsCorLibRandom::from_seed(seed)),
    }
}
