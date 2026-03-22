use std::num::Wrapping;
use std::ops::Range;

use anyhow::{bail, Result};

use crate::observation::Platform;

pub trait Prng {
    fn from_seed(seed: i32) -> Result<Self>
    where
        Self: Sized;
    fn gen_range(&mut self, range: Range<i32>) -> Result<i32>;
    fn gen_float(&mut self) -> Result<f64>;

    /// Equivalent to Next(2) == 1  (i.e. a fair coin flip).
    fn next_bool(&mut self) -> Result<bool> {
        Ok(self.gen_range(0..2)? == 1)
    }
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

        let difference: u32 = (Wrapping(range.end) - Wrapping(range.start)).0 as u32;
        if difference == 1u32 {
            Ok(range.start)
        } else {
            Ok((Wrapping(range.start) + Wrapping((self.gen() % difference) as i32)).0)
        }
    }

    fn gen_float(&mut self) -> Result<f64> {
        let a: f64 = (self.gen() >> 6) as f64;
        let b: f64 = (self.gen() >> 6) as f64;

        Ok((a * 134217728f64 + b) / 9007199254740992f64)
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
    fn from_seed(seed: i32) -> Result<Self> {
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
                if s.seed[i] < 0i32 {
                    s.seed[i] += i32::MAX;
                }
            }
        }

        Ok(s)
    }

    fn gen_range(&mut self, range: Range<i32>) -> Result<i32> {
        if range.is_empty() {
            bail!("Empty range parsed to MsCorLibRandom.gen_range().");
        }

        let difference: u32 = (Wrapping(range.end) - Wrapping(range.start)).0 as u32;
        if difference <= i32::MAX as u32 {
            Ok(range.start + (self.gen_float()? * difference as f64) as i32)
        } else {
            let mut sample: i32 = self.gen();
            if self.gen() % 2i32 == 0i32 {
                sample = -sample;
            }

            let mut sample: f64 = sample as f64;
            sample += (i32::MAX - 1i32) as f64;
            sample /= ((2u32 * i32::MAX as u32) - 1u32) as f64;

            Ok((Wrapping(range.start) + Wrapping((sample * difference as f64) as u32 as i32)).0)
        }
    }

    fn gen_float(&mut self) -> Result<f64> {
        Ok(self.gen() as f64 * (1f64 / i32::MAX as f64))
    }
}

pub fn get_prng(platform: Platform, seed: i32) -> Result<Box<dyn Prng>> {
    Ok(match platform {
        Platform::Switch => Box::new(Jkiss::from_seed(seed)?),
        Platform::PC => Box::new(MsCorLibRandom::from_seed(seed)?),
    })
}

// --- Stardew Valley 1.6 PRNG API ---
// CreateRandom(params double[] seeds): xxHash32 hash of the seeds array,
// cast to i32, used as seed for the platform-appropriate Knuth RNG.

const XXHASH_PRIME1: u32 = 2654435761;
const XXHASH_PRIME2: u32 = 2246822519;
const XXHASH_PRIME3: u32 = 3266489917;
const XXHASH_PRIME4: u32 = 668265263;
const XXHASH_PRIME5: u32 = 374761393;

/// xxHash32 over the raw little-endian bytes of a slice of values.
fn xxhash32(data: &[u8]) -> u32 {

    let len = data.len();
    let mut pos = 0usize;
    let mut h32: u32;

    // Helper: read a 4-byte little-endian u32 from data at offset
    let lane = |data: &[u8], offset: usize| {
        u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap())
    };

    if len >= 16 {
        let mut v1: u32 = 0u32.wrapping_add(XXHASH_PRIME1).wrapping_add(XXHASH_PRIME2);
        let mut v2: u32 = XXHASH_PRIME2;
        let mut v3: u32 = 0u32;
        let mut v4: u32 = 0u32.wrapping_sub(XXHASH_PRIME1);

        while pos + 16 <= len {
            v1 = v1.wrapping_add(lane(&data, pos).wrapping_mul(XXHASH_PRIME2)).rotate_left(13).wrapping_mul(XXHASH_PRIME1);
            pos += 4;
            v2 = v2.wrapping_add(lane(&data, pos).wrapping_mul(XXHASH_PRIME2)).rotate_left(13).wrapping_mul(XXHASH_PRIME1);
            pos += 4;
            v3 = v3.wrapping_add(lane(&data, pos).wrapping_mul(XXHASH_PRIME2)).rotate_left(13).wrapping_mul(XXHASH_PRIME1);
            pos += 4;
            v4 = v4.wrapping_add(lane(&data, pos).wrapping_mul(XXHASH_PRIME2)).rotate_left(13).wrapping_mul(XXHASH_PRIME1);
            pos += 4;
        }

        h32 = v1.rotate_left(1)
            .wrapping_add(v2.rotate_left(7))
            .wrapping_add(v3.rotate_left(12))
            .wrapping_add(v4.rotate_left(18));
    } else {
        h32 = XXHASH_PRIME5;
    }

    h32 = h32.wrapping_add(len as u32);

    while pos + 4 <= len {
        h32 = h32
            .wrapping_add(lane(&data, pos).wrapping_mul(XXHASH_PRIME3))
            .rotate_left(17)
            .wrapping_mul(XXHASH_PRIME4);
        pos += 4;
    }

    while pos < len {
        h32 = h32
            .wrapping_add((data[pos] as u32).wrapping_mul(XXHASH_PRIME5))
            .rotate_left(11)
            .wrapping_mul(XXHASH_PRIME1);
        pos += 1;
    }

    // Avalanche / finalisation
    h32 ^= h32 >> 15;
    h32 = h32.wrapping_mul(XXHASH_PRIME2);
    h32 ^= h32 >> 13;
    h32 = h32.wrapping_mul(XXHASH_PRIME3);
    h32 ^= h32 >> 16;

    h32
}

/// CreateRandom(params double[] seeds) — hash seeds → platform RNG.
///
/// Stardew Valley 1.6 converts each double seed to i32 via `(seed % i32::MAX) as i32`,
/// packs those as LE bytes, and feeds them to xxHash32.  The game does NOT hash the raw
/// f64 bytes.  See: Utility.CreateRandomSeed / HashUtility.GetDeterministicHashCode.
pub fn create_random(platform: Platform, seeds: &[f64]) -> Result<Box<dyn Prng>> {
    let data: Vec<u8> = seeds
        .iter()
        .flat_map(|&s| ((s % (i32::MAX as f64)) as i32).to_le_bytes())
        .collect();
    let hash = xxhash32(&data) as i32;
    get_prng(platform, hash)
}

/// CreateDaySaveRandom(seedA, seedB, seedC) =
///   CreateRandom(DaysPlayed, uniqueIDForThisGame / 2, seedA, seedB, seedC)
pub fn create_day_save_random(
    platform: Platform,
    days_played: u32,
    unique_id: u64,
    a: f64,
    b: f64,
    c: f64,
) -> Result<Box<dyn Prng>> {
    create_random(
        platform,
        &[days_played as f64, (unique_id / 2) as f64, a, b, c],
    )
}

/// CreateInitializationRandom() = CreateRandom(uniqueIDForThisGame, DaysPlayed)
pub fn create_initialisation_random(
    platform: Platform,
    unique_id: u64,
    days_played: u32,
) -> Result<Box<dyn Prng>> {
    create_random(platform, &[unique_id as f64, days_played as f64])
}
