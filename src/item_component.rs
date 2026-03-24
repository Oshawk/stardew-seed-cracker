use std::rc::Rc;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::codegen::{EligibleObject, ELIGIBLE_OBJECTS, ELIGIBLE_OBJECTS_SORTED};
use crate::traveling_merchant::{possible_prices, Item};

const ICON_SIZE: u16 = 16u16;
const ICONS_PER_ROW: u16 = 24u16;
const ICONS_FILE: &str = "./assets/springobjects.png";

macro_rules! icon_image {
    ($id:expr) => {
        match ($id) {
            Some(id) => html!(<figure class={ format!("image is-{}x{}", ICON_SIZE, ICON_SIZE) } style={ format!("background: url({}) -{}px -{}px", ICONS_FILE, (id % ICONS_PER_ROW) * ICON_SIZE, (id / ICONS_PER_ROW) * ICON_SIZE) } />),
            None => html!(<figure class={ format!("image is-{}x{}", ICON_SIZE, ICON_SIZE) } />)
        }
    };
}

#[derive(Clone, PartialEq, Properties)]
pub struct ItemProperties {
    pub index: usize,
    pub callback: Callback<(usize, Option<Item>)>,
}

#[derive(Default, Clone, PartialEq)]
struct ItemState {
    item_focus: bool,
    item_value: String,
    eligible_index: Option<u16>,
    price_focus: bool,
    price_value: Option<u16>,
    quantity_focus: bool,
    quantity_value: Option<u8>,
}

enum ItemAction {
    ItemFocus(bool),
    ItemInput(String),
    ItemSelect(u16), // eligible_index
    PriceFocus(bool),
    PriceSelect(u16),
    QuantityFocus(bool),
    QuantitySelect(u8),
}

impl Reducible for ItemState {
    type Action = ItemAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut s = (*self).clone();
        match action {
            ItemAction::ItemFocus(f) => s.item_focus = f,
            ItemAction::ItemInput(v) => s.item_value = v,
            ItemAction::ItemSelect(idx) => {
                s.item_value = ELIGIBLE_OBJECTS[idx as usize].name.to_string();
                s.eligible_index = Some(idx);
                s.price_value = None;
                s.quantity_value = None;
            }
            ItemAction::PriceFocus(f) => s.price_focus = f,
            ItemAction::PriceSelect(v) => s.price_value = Some(v),
            ItemAction::QuantityFocus(f) => s.quantity_focus = f,
            ItemAction::QuantitySelect(v) => s.quantity_value = Some(v),
        }
        Rc::new(s)
    }
}

#[component]
pub fn ItemComponent(props: &ItemProperties) -> Html {
    let state = use_reducer(ItemState::default);

    {
        let callback = props.callback.clone();
        let prop_index = props.index;
        let deps = (state.eligible_index, state.price_value, state.quantity_value);
        use_effect_with(deps, move |(eligible_index, price_value, quantity_value)| {
            let item = match (*eligible_index, *price_value, *quantity_value) {
                (Some(eligible_index), Some(price), Some(quantity)) => Some(Item {
                    eligible_index,
                    price,
                    quantity,
                }),
                _ => None,
            };
            callback.emit((prop_index, item));
        });
    }

    let dispatch = state.dispatcher();

    let on_item_focus = {
        let dispatch = dispatch.clone();
        Callback::from(move |_| dispatch.dispatch(ItemAction::ItemFocus(true)))
    };
    let on_item_blur = {
        let dispatch = dispatch.clone();
        Callback::from(move |_| dispatch.dispatch(ItemAction::ItemFocus(false)))
    };
    let on_item_input = {
        let dispatch = dispatch.clone();
        Callback::from(move |e: InputEvent| {
            let value = e.target_unchecked_into::<HtmlInputElement>().value();
            dispatch.dispatch(ItemAction::ItemInput(value));
        })
    };

    let on_price_focus = {
        let dispatch = dispatch.clone();
        Callback::from(move |_| dispatch.dispatch(ItemAction::PriceFocus(true)))
    };
    let on_price_blur = {
        let dispatch = dispatch.clone();
        Callback::from(move |_| dispatch.dispatch(ItemAction::PriceFocus(false)))
    };

    let on_quantity_focus = {
        let dispatch = dispatch.clone();
        Callback::from(move |_| dispatch.dispatch(ItemAction::QuantityFocus(true)))
    };
    let on_quantity_blur = {
        let dispatch = dispatch.clone();
        Callback::from(move |_| dispatch.dispatch(ItemAction::QuantityFocus(false)))
    };

    let item_dropdown_class = if state.item_focus { "dropdown is-active" } else { "dropdown" };
    let price_dropdown_class = if state.eligible_index.is_some() && state.price_focus {
        "dropdown is-active"
    } else {
        "dropdown"
    };
    let quantity_dropdown_class = if state.eligible_index.is_some() && state.quantity_focus {
        "dropdown is-active"
    } else {
        "dropdown"
    };

    let price_label = match state.price_value {
        Some(v) => format!("{}g", v),
        None => "Price".to_string(),
    };
    let quantity_label = match state.quantity_value {
        Some(v) => format!("x{}", v),
        None => "Quantity".to_string(),
    };

    // Get sprite ID for currently selected item
    let sprite_id: Option<u16> = state
        .eligible_index
        .map(|idx| ELIGIBLE_OBJECTS[idx as usize].id);

    let filtered_items: Vec<Html> = ELIGIBLE_OBJECTS_SORTED
        .iter()
        .filter_map(|sorted_idx: &u16| {
            let elig_idx = *sorted_idx;
            let info: &EligibleObject = &ELIGIBLE_OBJECTS[elig_idx as usize];
            if info
                .name
                .to_lowercase()
                .starts_with(&state.item_value.to_lowercase())
            {
                let dispatch = dispatch.clone();
                let obj_id = info.id;
                Some(html! {
                    <a class="dropdown-item" onmousedown={Callback::from(move |_| dispatch.dispatch(ItemAction::ItemSelect(elig_idx)))}>
                        <div class="columns is-vcentered">
                            <div class="column is-narrow">
                                { icon_image!(Some(obj_id)) }
                            </div>
                            <div class="column">
                                { info.name }
                            </div>
                        </div>
                    </a>
                })
            } else {
                None
            }
        })
        .take(5)
        .collect();

    let price_options: Html = match state.eligible_index {
        Some(elig_idx) => {
            let prices = possible_prices(elig_idx);
            prices
                .into_iter()
                .map(|price| {
                    let dispatch = dispatch.clone();
                    html! {
                        <a class="dropdown-item" onmousedown={Callback::from(move |_| dispatch.dispatch(ItemAction::PriceSelect(price)))}>
                            { format!("{}g", price) }
                        </a>
                    }
                })
                .collect()
        }
        None => html!(),
    };

    html! {
        <div class="field has-addons">
            <div class={item_dropdown_class} style="flex: 2">
                <div class="control">
                    <button class="button is-static">
                        { icon_image!(sprite_id) }
                    </button>
                </div>
                <div class="control is-expanded">
                    <div class="dropdown-trigger">
                        <input
                            class="input"
                            type="text"
                            placeholder="Item"
                            value={state.item_value.clone()}
                            onfocus={on_item_focus}
                            onblur={on_item_blur}
                            oninput={on_item_input}
                        />
                    </div>
                </div>
                <div class="dropdown-menu" style="width:100%">
                    <div class="dropdown-content">
                        { for filtered_items }
                    </div>
                </div>
            </div>
            <div class={price_dropdown_class} style="flex: 1">
                <div class="control is-expanded">
                    <div class="dropdown-trigger">
                        <button
                            class="button is-fullwidth is-justify-content-space-between"
                            disabled={state.eligible_index.is_none()}
                            onfocus={on_price_focus}
                            onblur={on_price_blur}
                        >
                            <span>{ price_label }</span>
                            <span class="material-symbols-outlined">{ "expand_more" }</span>
                        </button>
                    </div>
                </div>
                <div class="dropdown-menu" style="width:100%">
                    <div class="dropdown-content">
                        { price_options }
                    </div>
                </div>
            </div>
            <div class={quantity_dropdown_class} style="flex: 1">
                <div class="control is-expanded">
                    <div class="dropdown-trigger">
                        <button
                            class="button is-fullwidth is-justify-content-space-between"
                            disabled={state.eligible_index.is_none()}
                            onfocus={on_quantity_focus}
                            onblur={on_quantity_blur}
                        >
                            <span>{ quantity_label }</span>
                            <span class="material-symbols-outlined">{ "expand_more" }</span>
                        </button>
                    </div>
                </div>
                <div class="dropdown-menu" style="width:100%">
                    <div class="dropdown-content">
                        <a class="dropdown-item" onmousedown={{ let d = dispatch.clone(); Callback::from(move |_| d.dispatch(ItemAction::QuantitySelect(1u8))) }}>{ "x1" }</a>
                        <a class="dropdown-item" onmousedown={{ let d = dispatch.clone(); Callback::from(move |_| d.dispatch(ItemAction::QuantitySelect(5u8))) }}>{ "x5" }</a>
                    </div>
                </div>
            </div>
        </div>
    }
}
