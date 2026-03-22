use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::codegen::OBJECTS;
use crate::dropdown::DropdownSelect;
use crate::observation::{FishingContent, Observation, QuestContent, ResourceContent, Season};

// All fish IDs that appear in any season/NPC pool.
const FISH_IDS: &[u32] = &[
    128, 129, 130, 131, 136, 137, 138, 139, 141, 142, 143, 144, 145, 146, 147, 149, 150, 151, 699,
    702, 705,
];

// Ore items requested by Clint: "– [X]g on delivery  – Keep the ores after Clint inspects them."
const ORE_ITEMS: &[(u32, &str)] = &[
    (378, "Copper Ore"),
    (380, "Iron Ore"),
    (382, "Coal"),
    (384, "Gold Ore"),
];

// Wood/stone items requested by Robin: "– [X]g on delivery  (only)"
const WOOD_STONE_ITEMS: &[(u32, &str)] = &[(388, "Wood"), (390, "Stone")];

/// UI-facing quest type — named after the in-game bullet-point patterns the
/// user sees on the notice board, not the internal quest class names.
#[derive(Clone, Copy, PartialEq)]
pub enum QuestTypeUI {
    NoQuest,
    Fishing,
    MonsterHunt,
    OreGathering,
    WoodStoneGathering,
    ItemDelivery,
    GreetEveryone,
}

impl QuestTypeUI {
    fn label(self) -> &'static str {
        match self {
            QuestTypeUI::NoQuest => "No quest",
            QuestTypeUI::Fishing => "- [..]g reward\n- You get to keep the fish.",
            QuestTypeUI::MonsterHunt => "- [...]g reward.",
            QuestTypeUI::OreGathering => "- [...]g on delivery.\n- You can keep the ores [...].",
            QuestTypeUI::WoodStoneGathering => "- [...]g on delivery.",
            QuestTypeUI::ItemDelivery => "- [...]g on delivery.\n- [...] happy/thankful/pleased.",
            QuestTypeUI::GreetEveryone => "- Everyone will like you a little more.",
        }
    }

    fn key(self) -> &'static str {
        match self {
            QuestTypeUI::NoQuest => "no-quest",
            QuestTypeUI::Fishing => "fishing",
            QuestTypeUI::MonsterHunt => "monster-hunt",
            QuestTypeUI::OreGathering => "ore-gathering",
            QuestTypeUI::WoodStoneGathering => "wood-stone",
            QuestTypeUI::ItemDelivery => "item-delivery",
            QuestTypeUI::GreetEveryone => "greet-everyone",
        }
    }

    fn from_key(s: &str) -> Option<Self> {
        match s {
            "no-quest" => Some(QuestTypeUI::NoQuest),
            "fishing" => Some(QuestTypeUI::Fishing),
            "monster-hunt" => Some(QuestTypeUI::MonsterHunt),
            "ore-gathering" => Some(QuestTypeUI::OreGathering),
            "wood-stone" => Some(QuestTypeUI::WoodStoneGathering),
            "item-delivery" => Some(QuestTypeUI::ItemDelivery),
            "greet-everyone" => Some(QuestTypeUI::GreetEveryone),
            _ => None,
        }
    }
}

/// Data state for a single observation row — stored in the parent App so it
/// survives re-renders (adding / removing rows).
#[derive(Clone, PartialEq, Default)]
pub struct RowDisplayState {
    pub day_value: String,
    pub day: Option<u8>,
    pub season: Option<Season>,
    pub year_value: String,
    pub year: Option<u32>,
    pub quest_type: Option<QuestTypeUI>,
    pub fish_npc_demetrius: Option<bool>,
    pub fish_id: Option<u32>,
    pub resource_item_id: Option<u32>,
}

pub fn build_observation(s: &RowDisplayState) -> Option<Observation> {
    let day = s.day?;
    let season = s.season?;
    let year = s.year?;
    let quest_type = s.quest_type?;

    let days_played = (year - 1) * 112 + season.index() * 28 + day as u32;

    let quest_content = match quest_type {
        QuestTypeUI::NoQuest => QuestContent::None,
        QuestTypeUI::ItemDelivery => QuestContent::ItemDelivery,
        QuestTypeUI::GreetEveryone => QuestContent::Socialize,
        QuestTypeUI::MonsterHunt => QuestContent::SlayMonster,
        QuestTypeUI::Fishing => {
            let demetrius = s.fish_npc_demetrius?;
            let fish_id = s.fish_id?;
            QuestContent::Fishing(FishingContent { demetrius, fish_id })
        }
        QuestTypeUI::OreGathering | QuestTypeUI::WoodStoneGathering => {
            let item_id = s.resource_item_id?;
            QuestContent::ResourceCollection(ResourceContent { item_id })
        }
    };

    Some(Observation {
        days_played,
        day_of_month: day,
        season,
        quest_content,
    })
}

// ---------------------------------------------------------------------------
// Static option lists
// ---------------------------------------------------------------------------

fn season_options() -> Vec<(String, String)> {
    vec![
        ("spring".into(), "Spring".into()),
        ("summer".into(), "Summer".into()),
        ("fall".into(), "Fall".into()),
        ("winter".into(), "Winter".into()),
    ]
}

fn quest_type_options() -> Vec<(String, String)> {
    [
        QuestTypeUI::NoQuest,
        QuestTypeUI::Fishing,
        QuestTypeUI::OreGathering,
        QuestTypeUI::ItemDelivery,
        QuestTypeUI::WoodStoneGathering,
        QuestTypeUI::MonsterHunt,
        QuestTypeUI::GreetEveryone,
    ]
    .iter()
    .map(|qt| (qt.key().to_string(), qt.label().to_string()))
    .collect()
}

fn fish_npc_options() -> Vec<(String, String)> {
    vec![
        ("demetrius".into(), "Demetrius".into()),
        ("willy".into(), "Willy".into()),
    ]
}

fn fish_options() -> Vec<(String, String)> {
    let mut opts: Vec<(String, String)> = FISH_IDS
        .iter()
        .filter_map(|&id| {
            OBJECTS
                .get(&id)
                .map(|obj| (id.to_string(), obj.name.to_string()))
        })
        .collect();
    opts.sort_by(|a, b| a.1.cmp(&b.1));
    opts
}

fn ore_options() -> Vec<(String, String)> {
    ORE_ITEMS
        .iter()
        .map(|&(id, label)| (id.to_string(), label.to_string()))
        .collect()
}

fn wood_stone_options() -> Vec<(String, String)> {
    WOOD_STONE_ITEMS
        .iter()
        .map(|&(id, label)| (id.to_string(), label.to_string()))
        .collect()
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq, Properties)]
pub struct ObservationRowProps {
    pub display_state: RowDisplayState,
    pub on_change: Callback<RowDisplayState>,
    pub on_delete: Callback<()>,
}

#[component]
pub fn ObservationRow(props: &ObservationRowProps) -> Html {
    let s = &props.display_state;
    let on_change = props.on_change.clone();

    // ---- Day ----
    let on_day_input = {
        let s = s.clone();
        let on_change = on_change.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<HtmlInputElement>().value();
            let mut ns = s.clone();
            ns.day = v.parse::<u8>().ok().filter(|&d| d >= 1 && d <= 28);
            ns.day_value = v;
            on_change.emit(ns);
        })
    };

    // ---- Season ----
    let on_season = {
        let s = s.clone();
        let on_change = on_change.clone();
        Callback::from(move |key: String| {
            let mut ns = s.clone();
            ns.season = Season::from_key(&key);
            on_change.emit(ns);
        })
    };

    // ---- Year ----
    let on_year_input = {
        let s = s.clone();
        let on_change = on_change.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<HtmlInputElement>().value();
            let mut ns = s.clone();
            ns.year = v.parse::<u32>().ok().filter(|&y| y >= 1);
            ns.year_value = v;
            on_change.emit(ns);
        })
    };

    // ---- Quest type ----
    let on_quest_type = {
        let s = s.clone();
        let on_change = on_change.clone();
        Callback::from(move |key: String| {
            let mut ns = s.clone();
            ns.quest_type = QuestTypeUI::from_key(&key);
            // Reset content fields when quest type changes.
            ns.fish_npc_demetrius = None;
            ns.fish_id = None;
            ns.resource_item_id = None;
            on_change.emit(ns);
        })
    };

    // ---- Fish NPC ----
    let on_fish_npc = {
        let s = s.clone();
        let on_change = on_change.clone();
        Callback::from(move |key: String| {
            let mut ns = s.clone();
            ns.fish_npc_demetrius = match key.as_str() {
                "demetrius" => Some(true),
                "willy" => Some(false),
                _ => None,
            };
            on_change.emit(ns);
        })
    };

    // ---- Fish ----
    let on_fish = {
        let s = s.clone();
        let on_change = on_change.clone();
        Callback::from(move |key: String| {
            let mut ns = s.clone();
            ns.fish_id = key.parse::<u32>().ok();
            on_change.emit(ns);
        })
    };

    // ---- Resource item ----
    let on_resource = {
        let s = s.clone();
        let on_change = on_change.clone();
        Callback::from(move |key: String| {
            let mut ns = s.clone();
            ns.resource_item_id = key.parse::<u32>().ok();
            on_change.emit(ns);
        })
    };

    // ---- Content section (quest-type-specific fields) ----
    let content_section: Html = match s.quest_type {
        Some(QuestTypeUI::Fishing) => html! {
            <>
                <DropdownSelect
                    options={fish_npc_options()}
                    selected={s.fish_npc_demetrius.map(|d| if d { "demetrius".to_string() } else { "willy".to_string() })}
                    placeholder="NPC"
                    on_select={on_fish_npc}
                />
                <DropdownSelect
                    options={fish_options()}
                    selected={s.fish_id.map(|id| id.to_string())}
                    placeholder="Fish"
                    on_select={on_fish}
                />
            </>
        },

        Some(QuestTypeUI::OreGathering) => html! {
            <DropdownSelect
                options={ore_options()}
                selected={s.resource_item_id.map(|id| id.to_string())}
                placeholder="Ore"
                on_select={on_resource}
            />
        },

        Some(QuestTypeUI::WoodStoneGathering) => html! {
            <DropdownSelect
                options={wood_stone_options()}
                selected={s.resource_item_id.map(|id| id.to_string())}
                placeholder="Material"
                on_select={on_resource}
            />
        },

        _ => html! {},
    };

    let on_delete = {
        let cb = props.on_delete.clone();
        Callback::from(move |_| cb.emit(()))
    };

    html! {
        <div class="box mb-2 py-3">
            // Outer flex: fields on the left (wrapping), delete button pinned to the right.
            <div style="display:flex; align-items:center; gap:0.5rem">
                // Inner flex: all fields, allowed to wrap on narrow screens.
                <div style="display:flex; flex:1; flex-wrap:wrap; align-items:center; gap:0.5rem">
                    <input
                        class="input"
                        type="text"
                        inputmode="numeric"
                        pattern="[0-9]*"
                        placeholder="Day"
                        style="width:4rem"
                        value={s.day_value.clone()}
                        oninput={on_day_input}
                    />
                    <DropdownSelect
                        options={season_options()}
                        selected={s.season.map(|s| s.key().to_string())}
                        placeholder="Season"
                        on_select={on_season}
                    />
                    <input
                        class="input"
                        type="text"
                        inputmode="numeric"
                        pattern="[0-9]*"
                        placeholder="Year"
                        style="width:4.5rem"
                        value={s.year_value.clone()}
                        oninput={on_year_input}
                    />
                    <span class="tag is-light">{ "—" }</span>
                    <DropdownSelect
                        options={quest_type_options()}
                        selected={s.quest_type.map(|qt| qt.key().to_string())}
                        placeholder="Select quest..."
                        on_select={on_quest_type}
                    />
                    { content_section }
                </div>
                // Delete button — outside the wrapping flex, always pinned to the right.
                <button
                    class="delete is-medium"
                    style="flex-shrink:0"
                    onclick={on_delete}
                />
            </div>
        </div>
    }
}
