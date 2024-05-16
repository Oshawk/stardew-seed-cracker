use web_sys::HtmlInputElement;
use yew::prelude::*;

pub enum DateMessage {
    YearValue(String),
    SeasonFocus(bool),
    SeasonValue(u8),
    DayValue(String),
}

#[derive(Clone, PartialEq, Properties)]
pub struct DateProperties {
    pub callback: Callback<Option<i32>>,
}

pub struct DateComponent {
    year_value: Option<u16>,
    season_focus: bool,
    season_value: Option<u8>,
    day_value: Option<u8>,
}

impl Component for DateComponent {
    type Message = DateMessage;
    type Properties = DateProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            year_value: None,
            season_focus: false,
            season_value: None,
            day_value: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::YearValue(year_string) => match year_string.parse::<u16>() {
                Ok(year_int) => {
                    if year_int > 0u16 {
                        self.year_value = Some(year_int);
                    } else {
                        self.year_value = None;
                    }
                }
                Err(_) => {
                    self.year_value = None;
                }
            },
            Self::Message::SeasonFocus(focus) => {
                self.season_focus = focus;
            }
            Self::Message::SeasonValue(season) => {
                if (1u8..=4u8).contains(&season) {
                    self.season_value = Some(season);
                } else {
                    self.season_value = None;
                }
            }
            Self::Message::DayValue(day_string) => match day_string.parse::<u8>() {
                Ok(day_int) => {
                    if (1u8..=28u8).contains(&day_int) {
                        self.day_value = Some(day_int);
                    } else {
                        self.day_value = None;
                    }
                }
                Err(_) => {
                    self.day_value = None;
                }
            },
        }

        match (self.year_value, self.season_value, self.day_value) {
            (Some(year), Some(season), Some(day)) => {
                ctx.props().callback.emit(Some(
                    28i32 * 4i32 * (year as i32 - 1i32)
                        + 28i32 * (season as i32 - 1i32)
                        + day as i32,
                ));
            }
            _ => {
                ctx.props().callback.emit(None);
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="field has-addons">
                <div class="control" style="flex: 1">
                    <input class="input" type="text" placeholder="Year" value={ match self.year_value { Some(year) => year.to_string(), None => "".to_string() } } oninput={ ctx.link().callback(|event: InputEvent| Self::Message::YearValue(event.target_unchecked_into::<HtmlInputElement>().value())) } />
                </div>
                <div class={ if self.season_focus { "dropdown is-active" } else { "dropdown" } } style="flex: 1">
                    <div class="control is-expanded">
                        <div class="dropdown-trigger">
                            <button class="button is-fullwidth is-justify-content-space-between" onfocus={ ctx.link().callback(|_| Self::Message::SeasonFocus(true)) } onblur={ ctx.link().callback(|_| Self::Message::SeasonFocus(false)) }>
                                <span>{ match self.season_value { Some(value) => match value { 1u8 => "Spring".to_string(), 2u8 => "Summer".to_string(), 3u8 => "Fall".to_string(), 4u8 => "Winter".to_string(), _ => "ERROR".to_string() }, None => "Season".to_string() } }</span>
                                <span class="material-symbols-outlined">{ "expand_more" }</span>
                            </button>
                        </div>
                    </div>
                    <div class="dropdown-menu" style="width:100%">
                        <div class="dropdown-content">
                            <a class="dropdown-item" onmousedown={ ctx.link().callback(move |_| Self::Message::SeasonValue(1u8)) }>{ "Spring" }</a>
                            <a class="dropdown-item" onmousedown={ ctx.link().callback(move |_| Self::Message::SeasonValue(2u8)) }>{ "Summer" }</a>
                            <a class="dropdown-item" onmousedown={ ctx.link().callback(move |_| Self::Message::SeasonValue(3u8)) }>{ "Fall" }</a>
                            <a class="dropdown-item" onmousedown={ ctx.link().callback(move |_| Self::Message::SeasonValue(4u8)) }>{ "Winter" }</a>
                        </div>
                    </div>
                </div>
                <div class="control" style="flex: 1">
                    <input class="input" type="text" placeholder="Day" value={ match self.day_value { Some(day) => day.to_string(), None => "".to_string() } } oninput={ ctx.link().callback(|event: InputEvent| Self::Message::DayValue(event.target_unchecked_into::<HtmlInputElement>().value())) } />
                </div>
            </div>
        }
    }
}
