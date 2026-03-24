use std::cmp::max;
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::codegen::{
    ObjectClass, ELIGIBLE_OBJECTS, OBJECT_ENUMERATION, TOTAL_ELIGIBLE, TOTAL_OBJECTS,
};
use crate::prng::{get_prng, Prng};

pub struct GeneratedStock {
    pub items: [GeneratedItem; STOCK_QUANTITY],
}

pub struct GeneratedItem {
    pub eligible_index: u16,
    pub price: u16,
    pub quantity: u8,
}

/// Forward-simulate the travelling cart stock for a given platform and shop seed.
pub fn generate_stock(platform: Platform, seed: i32) -> GeneratedStock {
    let mut prng = get_prng(platform, seed);

    // Phase A: assign a Next() key to every eligible object, take the 10 smallest.
    let mut keyed: Vec<(i32, u16)> = Vec::with_capacity(TOTAL_ELIGIBLE);
    for pos in 0..TOTAL_OBJECTS {
        let key = prng.next();
        if let ObjectClass::FullyEligible(idx) = OBJECT_ENUMERATION[pos].class {
            keyed.push((key, idx));
        }
    }
    keyed.sort_by_key(|&(key, _)| key);

    // Phase B: price and quantity rolls, one per slot in sort-key (= UI slot) order.
    let items = std::array::from_fn(|slot| {
        let set_idx = prng.next_max(10);
        let mult_idx = prng.next_max(3);
        let qty_roll = prng.next_double();

        let elig_idx = keyed[slot].1;
        let base_price = ELIGIBLE_OBJECTS[elig_idx as usize].price;
        let set_price = (set_idx as u16 + 1) * 100;
        let multiplied = base_price * [3u16, 4u16, 5u16][mult_idx as usize];
        let price = max(set_price, multiplied);
        let quantity = if qty_roll < 0.1 { 5 } else { 1 };

        GeneratedItem { eligible_index: elig_idx, price, quantity }
    });

    GeneratedStock { items }
}

pub const STOCK_QUANTITY: usize = 10;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Platform {
    PC,
    Switch,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Item {
    pub eligible_index: u16,
    pub price: u16,
    pub quantity: u8,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TravelingMerchant {
    pub platform: Platform,
    pub stock: [Item; STOCK_QUANTITY],
    /// Pre-computed: observed_slots[eligible_index] = Some(ui_slot) if this item is in the stock.
    pub observed_slots: Vec<Option<u8>>,
}

impl TravelingMerchant {
    pub fn new(platform: Platform, stock: [Item; STOCK_QUANTITY]) -> Self {
        let mut observed_slots: Vec<Option<u8>> = vec![None; TOTAL_ELIGIBLE];
        for (slot, item) in stock.iter().enumerate() {
            observed_slots[item.eligible_index as usize] = Some(slot as u8);
        }
        Self {
            platform,
            stock,
            observed_slots,
        }
    }

    pub fn seed_valid(&self, seed: i32) -> bool {
        let mut prng: Box<dyn Prng> = get_prng(self.platform, seed);

        // Phase A: Object shuffle with early rejection (807 next() calls)
        let mut min_non_obs_eligible_key: i32 = i32::MAX;
        let mut obs_keys: [i32; STOCK_QUANTITY] = [0; STOCK_QUANTITY];
        let mut obs_seen: [bool; STOCK_QUANTITY] = [false; STOCK_QUANTITY];
        let mut obs_count: u8 = 0;
        let mut max_obs_key: i32 = 0;

        for pos in 0..TOTAL_OBJECTS {
            let key = prng.next();
            match OBJECT_ENUMERATION[pos].class {
                ObjectClass::FullyEligible(elig_idx) => {
                    if let Some(slot) = self.observed_slots[elig_idx as usize] {
                        let s = slot as usize;

                        // Check against non-observed items
                        if min_non_obs_eligible_key < key {
                            return false;
                        }

                        // Check ordering against previously-seen observed items
                        for t in 0..STOCK_QUANTITY {
                            if obs_seen[t] {
                                if t < s && obs_keys[t] > key {
                                    return false; // earlier UI slot has larger key
                                }
                                if t > s && obs_keys[t] < key {
                                    return false; // later UI slot has smaller key
                                }
                            }
                        }

                        obs_keys[s] = key;
                        obs_seen[s] = true;
                        obs_count += 1;
                        if key > max_obs_key {
                            max_obs_key = key;
                        }
                    } else {
                        // Non-observed fully eligible item
                        if key < min_non_obs_eligible_key {
                            min_non_obs_eligible_key = key;
                        }
                        if key < max_obs_key {
                            return false;
                        }
                    }
                }
                ObjectClass::Intermediate | ObjectClass::Ineligible => {
                    // Just advance RNG, no checks needed
                }
            }
        }

        // Verify all 10 observed items were encountered
        if obs_count != STOCK_QUANTITY as u8 {
            return false;
        }

        // Final check: no non-observed eligible item beat the largest observed key
        // (already checked incrementally, but verify min_non_obs > max_obs)
        if min_non_obs_eligible_key < max_obs_key {
            return false;
        }

        // Phase B: Price and quantity verification (10 items x 3 RNG calls)
        // Items are processed in sort-key order (ascending), which matches UI slot order 0..9
        for slot in 0..STOCK_QUANTITY {
            let set_idx = prng.next_max(10);
            let mult_idx = prng.next_max(3);
            let qty_roll = prng.next_double();

            let item = &self.stock[slot];
            let base_price = ELIGIBLE_OBJECTS[item.eligible_index as usize].price;

            let set_price = (set_idx as u16 + 1) * 100; // [100, 200, ..., 1000]
            let multiplied = base_price * [3u16, 4u16, 5u16][mult_idx as usize];
            let expected_price = max(set_price, multiplied);
            let expected_qty: u8 = if qty_roll < 0.1 { 5 } else { 1 };

            if expected_price != item.price || expected_qty != item.quantity {
                return false;
            }
        }

        true
    }
}

pub fn possible_prices(eligible_index: u16) -> Vec<u16> {
    let base_price = ELIGIBLE_OBJECTS[eligible_index as usize].price;
    let mut prices: BTreeSet<u16> = BTreeSet::new();

    for set_idx in 0..10u16 {
        for mult_idx in 0..3usize {
            let set_price = (set_idx + 1) * 100;
            let multiplied = base_price * [3u16, 4u16, 5u16][mult_idx];
            prices.insert(max(set_price, multiplied));
        }
    }

    prices.into_iter().collect()
}
