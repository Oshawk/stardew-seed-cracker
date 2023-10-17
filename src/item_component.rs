use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::codegen::{ObjectInformation, OBJECT_INFORMATION, OBJECT_INFORMATION_SORTED};
use crate::traveling_merchant::{possible_prices, Item};

const ICON_SIZE: u16 = 16u16;
const ICONS_PER_ROW: u16 = 24u16;
const ICONS_FILE: &str = "springobjects.png";

macro_rules! icon_image {
    ($index:expr) => {
        match ($index) {
            Some(index) => html!(<figure class={ format!("image is-{}x{}", ICON_SIZE, ICON_SIZE) } style={ format!("background: url({}) -{}px -{}px", ICONS_FILE, (index % ICONS_PER_ROW) * ICON_SIZE, (index / ICONS_PER_ROW) * ICON_SIZE) } />),
            None => html!(<figure class={ format!("image is-{}x{}", ICON_SIZE, ICON_SIZE) } />)
        }
    };
}

pub enum ItemMessage {
    ItemFocus(bool),
    ItemValue(String),
    ItemIndex(u16),
    PriceFocus(bool),
    PriceValue(u16),
    QuantityFocus(bool),
    QuantityValue(u8),
}

#[derive(Clone, PartialEq, Properties)]
pub struct ItemProperties {
    pub index: usize,
    pub callback: Callback<(usize, Option<Item>)>,
}

pub struct ItemComponent {
    item_focus: bool,
    item_value: String,
    item_index: Option<u16>,
    price_focus: bool,
    price_value: Option<u16>,
    quantity_focus: bool,
    quantity_value: Option<u8>,
}

impl Component for ItemComponent {
    type Message = ItemMessage;
    type Properties = ItemProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            item_focus: false,
            item_value: "".to_string(),
            item_index: None,
            price_focus: false,
            price_value: None,
            quantity_focus: false,
            quantity_value: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::ItemFocus(focus) => {
                self.item_focus = focus;
            }
            Self::Message::ItemValue(value) => {
                self.item_value = value;
            }
            Self::Message::ItemIndex(index) => {
                self.item_value = OBJECT_INFORMATION.get(&index).unwrap().name.to_string();
                self.item_index = Some(index);
                self.price_value = None;
                self.quantity_value = None;
            }
            Self::Message::PriceFocus(focus) => {
                self.price_focus = focus;
            }
            Self::Message::PriceValue(value) => {
                self.price_value = Some(value);
            }
            Self::Message::QuantityFocus(focus) => {
                self.quantity_focus = focus;
            }
            Self::Message::QuantityValue(value) => {
                self.quantity_value = Some(value);
            }
        }

        match (self.item_index, self.price_value, self.quantity_value) {
            (Some(index), Some(price), Some(quantity)) => {
                ctx.props().callback.emit((
                    ctx.props().index,
                    Some(Item {
                        index,
                        price,
                        quantity,
                    }),
                ));
            }
            _ => {
                ctx.props().callback.emit((ctx.props().index, None));
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="field has-addons">
                <div class={ if self.item_focus { "dropdown is-active" } else { "dropdown" } } style="flex: 2">
                    <div class="control">
                        <button class="button is-static">
                            { icon_image!(self.item_index) }
                        </button>
                    </div>
                    <div class="control is-expanded">
                        <div class="dropdown-trigger">
                            <input class="input" type="text" placeholder="Item" value={ self.item_value.clone() } onfocus={ ctx.link().callback(|_| Self::Message::ItemFocus(true)) } onblur={ ctx.link().callback(|_| Self::Message::ItemFocus(false)) } oninput={ ctx.link().callback(|event: InputEvent| Self::Message::ItemValue(event.target_unchecked_into::<HtmlInputElement>().value())) }/>
                        </div>
                    </div>
                    <div class="dropdown-menu" style="width:100%">
                        <div class="dropdown-content">
                            {
                                OBJECT_INFORMATION_SORTED.iter().filter_map(|index: &u16| {
                                    let object_information: &ObjectInformation = OBJECT_INFORMATION.get(index).unwrap();
                                    if object_information.name.to_lowercase().starts_with(&self.item_value.to_lowercase()) {
                                        Some(html! {
                                            <a class="dropdown-item" onmousedown={ ctx.link().callback(|_| Self::Message::ItemIndex(*index)) }>
                                                <div class="columns is-vcentered">
                                                    <div class="column is-narrow">
                                                        { icon_image!(Some(*index)) }
                                                    </div>
                                                    <div class="column">
                                                        { object_information.name }
                                                    </div>
                                                </div>
                                            </a>
                                        })
                                    } else {
                                        None
                                    }
                                }).take(5).collect::<Html>()
                            }
                        </div>
                    </div>
                </div>
                <div class={ if self.item_index.is_some() && self.price_focus { "dropdown is-active" } else { "dropdown" } } style="flex: 1">
                    <div class="control is-expanded">
                        <div class="dropdown-trigger">
                            <button class="button is-fullwidth is-justify-content-space-between" disabled={ self.item_index.is_none() } onfocus={ ctx.link().callback(|_| Self::Message::PriceFocus(true)) } onblur={ ctx.link().callback(|_| Self::Message::PriceFocus(false)) }>
                                <span>{ match self.price_value { Some(value) => format!("{}g", value), None => "Price".to_string() } }</span>
                                <span class="material-symbols-outlined">{ "expand_more" }</span>
                            </button>
                        </div>
                    </div>
                    <div class="dropdown-menu" style="width:100%">
                        <div class="dropdown-content">
                            {
                                match self.item_index {
                                    Some(index) => {
                                        let prices: Vec<u16> = possible_prices(index).unwrap();
                                        prices.into_iter().map(|price: u16| {
                                            html!(<a class="dropdown-item" onmousedown={ ctx.link().callback(move |_| Self::Message::PriceValue(price)) }>{ format!("{}g", price) }</a> )
                                        }).collect::<Html>()
                                    }
                                    None => html!()
                                }
                            }
                        </div>
                    </div>
                </div>
                <div class={ if self.item_index.is_some() && self.quantity_focus { "dropdown is-active" } else { "dropdown" } } style="flex: 1">
                    <div class="control is-expanded">
                        <div class="dropdown-trigger">
                            <button class="button is-fullwidth is-justify-content-space-between" disabled={ self.item_index.is_none() } onfocus={ ctx.link().callback(|_| Self::Message::QuantityFocus(true)) } onblur={ ctx.link().callback(|_| Self::Message::QuantityFocus(false)) }>
                                <span>{ match self.quantity_value { Some(value) => format!("x{}", value), None => "Quantity".to_string() } }</span>
                                <span class="material-symbols-outlined">{ "expand_more" }</span>
                            </button>
                        </div>
                    </div>
                    <div class="dropdown-menu" style="width:100%">
                        <div class="dropdown-content">
                            <a class="dropdown-item" onmousedown={ ctx.link().callback(move |_| Self::Message::QuantityValue(1u8)) }>{ "x1" }</a>
                            <a class="dropdown-item" onmousedown={ ctx.link().callback(move |_| Self::Message::QuantityValue(5u8)) }>{ "x5" }</a>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}
