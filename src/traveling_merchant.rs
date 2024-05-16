use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::cmp::max;
use std::collections::HashSet;

use crate::codegen::{
    ObjectInformation, FAST_EXCLUDE, FIRST_FILTER, OBJECT_INFORMATION, SECOND_FILTER,
};
use crate::prng::{get_prng, Prng};

pub const STOCK_QUANTITY: usize = 10usize;

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Platform {
    PC,
    Switch,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Item {
    pub index: u16,
    pub price: u16,
    pub quantity: u8,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TravelingMerchant {
    pub platform: Platform,
    pub stock: [Item; STOCK_QUANTITY],
}

impl TravelingMerchant {
    pub fn seed_valid(&self, seed: i32) -> Result<bool> {
        let mut prng: Box<dyn Prng> = get_prng(self.platform, seed)?;

        let mut used_indexes: HashSet<u16> = HashSet::new();

        for item in self.stock.iter() {
            let mut index = prng.gen_range(2i32..790i32)? as u16;

            let mut fast_exclude_index = index;
            loop {
                fast_exclude_index = *FAST_EXCLUDE
                    .get(&fast_exclude_index)
                    .context("Issue with fast exclude.")?;
                if !used_indexes.contains(&fast_exclude_index) {
                    break;
                }
            }

            if fast_exclude_index != item.index {
                return Ok(false);
            }

            loop {
                index += 1u16;
                index %= 790u16;

                if FIRST_FILTER.contains(&index) {
                    continue;
                }

                let constant_multiplier: u16;
                let variable_multiplier: u16;
                let quantity_decider: f64;
                match self.platform {
                    Platform::PC => {
                        if SECOND_FILTER.contains(&index) {
                            continue;
                        }

                        constant_multiplier = prng.gen_range(1i32..11i32)? as u16;
                        variable_multiplier = prng.gen_range(3i32..6i32)? as u16;
                        quantity_decider = prng.gen_float()?;
                    }
                    Platform::Switch => {
                        // Switch edition moves these before the checks.
                        constant_multiplier = prng.gen_range(1i32..11i32)? as u16;
                        variable_multiplier = prng.gen_range(3i32..6i32)? as u16;
                        quantity_decider = prng.gen_float()?;

                        if SECOND_FILTER.contains(&index) {
                            continue;
                        }
                    }
                }

                if !used_indexes.insert(index) {
                    continue;
                }

                let quantity = if quantity_decider < 0.1f64 { 5u8 } else { 1u8 };

                if quantity != item.quantity {
                    return Ok(false);
                }

                let object_information: &ObjectInformation = OBJECT_INFORMATION
                    .get(&index)
                    .context("Issue with object information.")?;

                let price: u16 = max(
                    100u16 * constant_multiplier,
                    object_information.price * variable_multiplier,
                );

                if price != item.price {
                    return Ok(false);
                }

                break;
            }
        }

        Ok(true)
    }
}

pub fn possible_prices(item_index: u16) -> Result<Vec<u16>> {
    let object_information: &ObjectInformation = OBJECT_INFORMATION
        .get(&item_index)
        .context("Issue with object information.")?;
    let minimum: u16 = max(100u16 * 1u16, object_information.price * 3u16);
    let mut prices: Vec<u16> = Vec::new();

    for constant_multiplier in 1u16..11u16 {
        let price: u16 = 100u16 * constant_multiplier;
        if price >= minimum && !prices.contains(&price) {
            prices.push(price)
        }
    }

    for variable_multiplier in 3u16..6u16 {
        let price: u16 = object_information.price * variable_multiplier;
        if price >= minimum && !prices.contains(&price) {
            prices.push(price)
        }
    }

    prices.sort();

    Ok(prices)
}
