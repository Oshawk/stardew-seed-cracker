use crate::game_data::{resource_type_to_items, FISHING_POOLS};
use crate::observation::{FishingContent, Observation, Platform, QuestContent, ResourceContent};
use crate::prng::{create_day_save_random, create_initialisation_random};

/// Returns true if `id` is consistent with every observation in the slice.
/// Observations should be pre-sorted by `pass_rate()` ascending for best performance.
pub fn check_all(platform: Platform, id: u64, observations: &[Observation]) -> bool {
    observations
        .iter()
        .all(|obs| check_observation(platform, id, obs))
}

/// Returns true if candidate `id` is consistent with a single observation.
pub fn check_observation(platform: Platform, id: u64, obs: &Observation) -> bool {
    check_type(platform, id, obs) && check_content(platform, id, obs)
}

fn check_type(platform: Platform, id: u64, obs: &Observation) -> bool {
    let mut rng = match create_day_save_random(
        platform,
        obs.days_played,
        id,
        100.0,
        obs.days_played as f64 * 777.0,
        0.0,
    ) {
        Ok(r) => r,
        Err(_) => return false,
    };

    let d = match rng.gen_float() {
        Ok(v) => v,
        Err(_) => return false,
    };

    match &obs.quest_content {
        // d in [0.08, 0.5): conservative — doesn't assume mineEntered.
        // If mine was entered and d in [0.08, 0.2), it may have been SlayMonster,
        // but without mine state we accept the whole [0.08, 0.5) range for None.
        QuestContent::None => d >= 0.08 && d < 0.5,

        // d in [0.0, 0.08)
        QuestContent::ResourceCollection(_) => d < 0.08,

        // d in [0.5, 0.6)
        QuestContent::Fishing(_) => d >= 0.5 && d < 0.6,

        // d in [0.6, 0.66) on a Monday (no active SocializeQuest)
        QuestContent::Socialize => d >= 0.6 && d < 0.66 && obs.is_monday(),

        // d in [0.6, 1.0) — conservative: accepts both Monday variants.
        // On Monday with d in [0.6, 0.66), it would be Socialize if no SocializeQuest
        // was active, or ItemDelivery if one was. We accept both here.
        QuestContent::ItemDelivery => d >= 0.6,

        // d in [0.08, 0.2) and DaysPlayed > 5 (mineEntered implied, unknown)
        QuestContent::SlayMonster => d >= 0.08 && d < 0.2 && obs.days_played > 5,
    }
}

fn check_content(platform: Platform, id: u64, obs: &Observation) -> bool {
    match &obs.quest_content {
        QuestContent::None
        | QuestContent::ItemDelivery
        | QuestContent::Socialize
        | QuestContent::SlayMonster => true,

        QuestContent::Fishing(c) => check_fishing(platform, id, obs, c),
        QuestContent::ResourceCollection(c) => check_resource(platform, id, obs, c),
    }
}

fn check_fishing(
    platform: Platform,
    id: u64,
    obs: &Observation,
    content: &FishingContent,
) -> bool {
    let mut rng = match create_initialisation_random(platform, id, obs.days_played) {
        Ok(r) => r,
        Err(_) => return false,
    };

    // sub_pool: false = Demetrius pool, true = Willy pool
    let sub_pool = match rng.next_bool() {
        Ok(v) => v,
        Err(_) => return false,
    };

    // content.demetrius == true means Demetrius was observed (sub_pool == false)
    if sub_pool == content.demetrius {
        // sub_pool=true → Willy, content.demetrius=true → mismatch
        // sub_pool=false → Demetrius, content.demetrius=false → mismatch
        return false;
    }

    let season_idx = obs.season.index() as usize;
    let pool = FISHING_POOLS[season_idx][sub_pool as usize];
    let pool_len = pool.len() as i32;

    let fish_idx = match rng.gen_range(0..pool_len) {
        Ok(v) => v as usize,
        Err(_) => return false,
    };

    pool[fish_idx] == content.fish_id
}

fn check_resource(
    platform: Platform,
    id: u64,
    obs: &Observation,
    content: &ResourceContent,
) -> bool {
    let mut rng = match create_initialisation_random(platform, id, obs.days_played) {
        Ok(r) => r,
        Err(_) => return false,
    };

    // resourceType = Next(6) * 2 → value in {0, 2, 4, 6, 8, 10}
    let resource_type = match rng.gen_range(0..6) {
        Ok(v) => v * 2,
        Err(_) => return false,
    };

    // Consume the dummy loop: dummyCount = Next(1, 100), then Next() × dummyCount
    let dummy_count = match rng.gen_range(1..100) {
        Ok(v) => v,
        Err(_) => return false,
    };
    for _ in 0..dummy_count {
        // Bare Next() call — generates an internal sample in [0, i32::MAX)
        if rng.gen_range(0..i32::MAX).is_err() {
            return false;
        }
    }

    // Check if observed item matches any possible item for this resource_type
    let possible = resource_type_to_items(resource_type);
    possible.contains(&content.item_id)
}
