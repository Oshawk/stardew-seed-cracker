use crate::codegen::{
    HASH_CART_COFFEE_BEAN, HASH_CART_FEZ, HASH_CART_RARECROW, HASH_CART_RETRO_CATALOGUE,
    HASH_TRAVELER_SKILL_BOOK,
};
use crate::prng::get_prng;
use crate::traveling_merchant::Platform;
use crate::xxhash::synced_random_seed;

/// Merchant visits on these days of each 28-day month.
const MERCHANT_DAYS: [i32; 8] = [5, 7, 12, 14, 19, 21, 26, 28];

/// Conditional items that can disambiguate two UIDs.
/// (key_hash, probability, name, season_filter)
/// season_filter: None = any season, Some([fall, winter]) = Fall/Winter only
struct ConditionalItem {
    key_hash: i32,
    probability: f64,
    name: &'static str,
    fall_winter_only: bool,
}

const CONDITIONAL_ITEMS: &[ConditionalItem] = &[
    ConditionalItem {
        key_hash: HASH_CART_FEZ,
        probability: 0.1,
        name: "Red Fez",
        fall_winter_only: false,
    },
    ConditionalItem {
        key_hash: HASH_CART_RETRO_CATALOGUE,
        probability: 0.1,
        name: "Retro Catalogue",
        fall_winter_only: false,
    },
    ConditionalItem {
        key_hash: HASH_TRAVELER_SKILL_BOOK,
        probability: 0.05,
        name: "Skill Book",
        fall_winter_only: false,
    },
    ConditionalItem {
        key_hash: HASH_CART_COFFEE_BEAN,
        probability: 0.25,
        name: "Coffee Bean",
        fall_winter_only: true,
    },
    ConditionalItem {
        key_hash: HASH_CART_RARECROW,
        probability: 0.4,
        name: "Rarecrow",
        fall_winter_only: true,
    },
];

pub struct DisambiguationResult {
    pub season: u8,
    pub day_of_month: i32,
    pub year: u16,
    pub item_name: &'static str,
    pub present_uid: u64,
    pub absent_uid: u64,
}

impl DisambiguationResult {
    pub fn season_name(&self) -> &'static str {
        match self.season {
            0 => "Spring",
            1 => "Summer",
            2 => "Fall",
            3 => "Winter",
            _ => "Unknown",
        }
    }
}

/// Given a k value (uid/2), find the earliest future merchant visit where
/// a conditional item differs between uid=2k and uid=2k+1.
pub fn disambiguate(
    k: u64,
    current_days_played: i32,
    platform: Platform,
) -> Option<DisambiguationResult> {
    let uid_even = 2 * k;
    let uid_odd = 2 * k + 1;

    // Search up to 10 years of future merchant visits
    let max_days = current_days_played + 28 * 4 * 10;

    let mut days_played = current_days_played;
    while days_played <= max_days {
        let day_of_month = ((days_played - 1) % 28) + 1;

        if !MERCHANT_DAYS.contains(&day_of_month) {
            days_played += 1;
            continue;
        }

        // Compute season (0-indexed: 0=Spring, 1=Summer, 2=Fall, 3=Winter)
        let season = (((days_played - 1) / 28) % 4) as u8;
        let year = (((days_played - 1) / (28 * 4)) + 1) as u16;
        let is_fall_winter = season >= 2;

        for item in CONDITIONAL_ITEMS {
            if item.fall_winter_only && !is_fall_winter {
                continue;
            }

            let seed_even = synced_random_seed(item.key_hash, uid_even, days_played);
            let seed_odd = synced_random_seed(item.key_hash, uid_odd, days_played);

            let mut rng_even = get_prng(platform, seed_even);
            let mut rng_odd = get_prng(platform, seed_odd);

            let present_even = rng_even.next_double() < item.probability;
            let present_odd = rng_odd.next_double() < item.probability;

            if present_even != present_odd {
                let (present_uid, absent_uid) = if present_even {
                    (uid_even, uid_odd)
                } else {
                    (uid_odd, uid_even)
                };

                return Some(DisambiguationResult {
                    season,
                    day_of_month,
                    year,
                    item_name: item.name,
                    present_uid,
                    absent_uid,
                });
            }
        }

        days_played += 1;
    }

    None
}
