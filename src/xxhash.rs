use xxhash_rust::xxh32::xxh32;

/// Compute the shop RNG seed for a given days_played and uid.
///
/// Matches the game's `Utility.CreateRandomSeed(DaysPlayed, uniqueIDForThisGame / 2)`:
/// `xxHash32(pack_le_bytes([days_played % 2147483647, (uid/2) % 2147483647, 0, 0, 0]))`
pub fn shop_seed(days_played: i32, uid: u64) -> i32 {
    let dp = (days_played as f64 % 2147483647.0) as i32;
    let k = ((uid / 2) as f64 % 2147483647.0) as i32;
    let mut buf = [0u8; 20];
    buf[0..4].copy_from_slice(&dp.to_le_bytes());
    buf[4..8].copy_from_slice(&k.to_le_bytes());
    // bytes 8..20 are already zero
    xxh32(&buf, 0) as i32
}

/// Compute the SYNCED_RANDOM seed for a given key hash, uid, and days_played.
///
/// Matches the game's `CreateRandom(key_hash, uniqueIDForThisGame, DaysPlayed)`:
/// `xxHash32(pack_le_bytes([key_hash % 2147483647, uid % 2147483647, days_played % 2147483647, 0, 0]))`
///
/// Note: uses full uid (NOT uid/2), unlike shop_seed.
pub fn synced_random_seed(key_hash: i32, uid: u64, days_played: i32) -> i32 {
    let kh = (key_hash as f64 % 2147483647.0) as i32;
    let u = (uid as f64 % 2147483647.0) as i32;
    let dp = (days_played as f64 % 2147483647.0) as i32;
    let mut buf = [0u8; 20];
    buf[0..4].copy_from_slice(&kh.to_le_bytes());
    buf[4..8].copy_from_slice(&u.to_le_bytes());
    buf[8..12].copy_from_slice(&dp.to_le_bytes());
    // bytes 12..20 are already zero
    xxh32(&buf, 0) as i32
}
