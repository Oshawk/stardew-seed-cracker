use serde::{Deserialize, Serialize};

// Platform moved here from traveling_merchant.rs.
// Switch uses Jkiss as the backing Random; PC uses MsCorLibRandom.
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Platform {
    PC,
    Switch,
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Season {
    Spring,
    Summer,
    Fall,
    Winter,
}

impl Season {
    /// 0-based index: Spring=0, Summer=1, Fall=2, Winter=3.
    pub fn index(self) -> u32 {
        match self {
            Season::Spring => 0,
            Season::Summer => 1,
            Season::Fall => 2,
            Season::Winter => 3,
        }
    }

    /// Stable string key used to round-trip through `DropdownSelect`.
    pub fn key(self) -> &'static str {
        match self {
            Season::Spring => "spring",
            Season::Summer => "summer",
            Season::Fall => "fall",
            Season::Winter => "winter",
        }
    }

    pub fn from_key(s: &str) -> Option<Self> {
        match s {
            "spring" => Some(Season::Spring),
            "summer" => Some(Season::Summer),
            "fall" => Some(Season::Fall),
            "winter" => Some(Season::Winter),
            _ => None,
        }
    }
}

/// Content for a FishingQuest observation.
#[derive(Clone, Serialize, Deserialize)]
pub struct FishingContent {
    /// true = Demetrius pool, false = Willy pool.
    /// Note: next_bool() returning false selects Demetrius (sub_pool=false),
    /// and next_bool() returning true selects Willy (sub_pool=true).
    pub demetrius: bool,
    pub fish_id: u32,
}

/// Content for a ResourceCollectionQuest observation.
#[derive(Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    /// The item ID observed (e.g. 378=Copper Ore, 380=Iron Ore, 382=Coal,
    /// 384=Gold Ore, 388=Wood, 390=Stone).
    pub item_id: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum QuestContent {
    /// No quest today. Constrains d to [0.08, 0.5) (conservative — no mine state).
    None,
    /// Fishing quest with full content matching.
    Fishing(FishingContent),
    /// Resource collection quest with content matching.
    ResourceCollection(ResourceContent),
    /// Item delivery quest — type-check only (d in [0.6, 1.0)).
    ItemDelivery,
    /// Socialize quest — type-check only (d in [0.6, 0.66) on a Monday).
    Socialize,
    /// Slay monster quest — type-check only (d in [0.08, 0.2), days_played > 5).
    SlayMonster,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Observation {
    /// Total days played (1-indexed): (year-1)*112 + season.index()*28 + day_of_month.
    pub days_played: u32,
    /// Day within the season (1–28).
    pub day_of_month: u8,
    pub season: Season,
    pub quest_content: QuestContent,
}

impl Observation {
    /// Mondays are days 1, 8, 15, 22 of each season.
    pub fn is_monday(&self) -> bool {
        matches!(self.day_of_month, 1 | 8 | 15 | 22)
    }

    /// Estimated fraction of candidate IDs that PASS this observation's filter.
    /// Lower = more discriminating = should be checked first in the pipeline.
    pub fn pass_rate(&self) -> f64 {
        match &self.quest_content {
            QuestContent::None => 0.42,           // d in [0.08, 0.5) = 42% of range
            QuestContent::Fishing(_) => 0.10 / 9.0, // ~10% type rate × ~1/9 fish pool
            QuestContent::ResourceCollection(_) => 0.08 / 6.0, // ~8% type × 1/6 resource
            QuestContent::ItemDelivery => 0.40,   // d in [0.6, 1.0)
            QuestContent::Socialize => 0.015,     // d in [0.6, 0.66) × ~1/4 Mondays
            QuestContent::SlayMonster => 0.12,    // d in [0.08, 0.2)
        }
    }
}
