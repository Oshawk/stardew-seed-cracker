// Hardcoded game data from Stardew Valley 1.6 source files.
// Sources: FishingQuest.cs, ResourceCollectionQuest.cs, Utility.cs

// Fish pools per season and NPC.
// Index: [season: 0=Spring,1=Summer,2=Fall,3=Winter][npc: 0=Demetrius,1=Willy]
// Values are item IDs from Data/Objects.json.
pub const FISHING_POOLS: [[&[u32]; 2]; 4] = [
    // Spring
    [
        &[129, 131, 136, 137, 142, 143, 145, 147],
        &[129, 131, 136, 137, 142, 143, 145, 147, 702],
    ],
    // Summer
    [
        &[130, 136, 138, 142, 144, 145, 146, 149, 150],
        &[128, 130, 136, 138, 142, 144, 145, 146, 149, 150, 702],
    ],
    // Fall
    [
        &[129, 131, 136, 137, 139, 142, 143, 150],
        &[129, 131, 136, 137, 139, 142, 143, 150, 699, 702, 705],
    ],
    // Winter
    [
        &[130, 131, 136, 141, 144, 146, 147, 150, 151],
        &[130, 131, 136, 141, 143, 144, 146, 147, 151, 699, 702, 705],
    ],
];

/// Returns the possible item IDs for a given resourceType value (next(6) * 2).
/// Most types return a single item. Type 6 can return Gold Ore (384) or Copper Ore (378)
/// depending on mine level; without knowing mine state both are accepted.
pub fn resource_type_to_items(resource_type: i32) -> &'static [u32] {
    match resource_type {
        0 => &[378],       // Copper Ore (Clint)
        2 => &[380],       // Iron Ore (Clint)
        4 => &[382],       // Coal (Clint)
        6 => &[384, 378],  // Gold Ore if mine>40, else Copper Ore (Clint) — both accepted
        8 => &[388],       // Wood (Robin)
        10 => &[390],      // Stone (Robin)
        _ => &[],
    }
}

/// Returns true if the given item ID is served by Clint (false = Robin).
pub fn resource_item_is_clint(item_id: u32) -> bool {
    matches!(item_id, 378 | 380 | 382 | 384)
}

// Item delivery base pool (always present, game-state independent).
pub const DELIVERY_BASE: &[u32] = &[378, 66, 78, 80, 86, 152, 167, 153, 420];

// Seasonal additions to the item delivery pool.
pub const DELIVERY_SPRING: &[u32] =
    &[16, 18, 20, 22, 129, 131, 132, 136, 137, 142, 143, 145, 147, 148, 152, 167, 267];
pub const DELIVERY_SUMMER: &[u32] =
    &[128, 130, 132, 136, 138, 142, 144, 145, 146, 149, 150, 155, 396, 398, 402, 267];
pub const DELIVERY_FALL: &[u32] =
    &[404, 406, 408, 410, 129, 131, 132, 136, 137, 139, 140, 142, 143, 148, 150, 154, 155, 269];
pub const DELIVERY_WINTER: &[u32] =
    &[412, 414, 416, 418, 130, 131, 132, 136, 140, 141, 144, 146, 147, 150, 151, 154, 269];
