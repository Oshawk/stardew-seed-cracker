use std::rc::Rc;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Clone, PartialEq, Properties)]
pub struct DateProperties {
    pub callback: Callback<Option<i32>>,
}

#[derive(Default, Clone, PartialEq)]
struct DateState {
    day: Option<u8>,
    season: Option<u8>,
    season_focus: bool,
    year: Option<u16>,
}

impl DateState {
    fn computed_date(&self) -> Option<i32> {
        match (self.year, self.season, self.day) {
            (Some(y), Some(s), Some(d)) => {
                Some(28 * 4 * (y as i32 - 1) + 28 * (s as i32 - 1) + d as i32)
            }
            _ => None,
        }
    }
}

enum DateAction {
    DayInput(String),
    SeasonFocus(bool),
    SeasonSelect(u8),
    YearInput(String),
}

impl Reducible for DateState {
    type Action = DateAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut s = (*self).clone();
        match action {
            DateAction::DayInput(v) => {
                s.day = v.parse::<u8>().ok().filter(|&d| (1u8..=28u8).contains(&d));
            }
            DateAction::SeasonFocus(f) => s.season_focus = f,
            DateAction::SeasonSelect(v) => {
                s.season = (1u8..=4u8).contains(&v).then_some(v);
            }
            DateAction::YearInput(v) => {
                s.year = v.parse::<u16>().ok().filter(|&y| y > 0);
            }
        }
        Rc::new(s)
    }
}

#[component]
pub fn DateComponent(props: &DateProperties) -> Html {
    let state = use_reducer(DateState::default);

    {
        let callback = props.callback.clone();
        let computed = state.computed_date();
        use_effect_with(computed, move |date| {
            callback.emit(*date);
        });
    }

    let dispatch = state.dispatcher();

    let on_day_input = {
        let dispatch = dispatch.clone();
        Callback::from(move |e: InputEvent| {
            let value = e.target_unchecked_into::<HtmlInputElement>().value();
            dispatch.dispatch(DateAction::DayInput(value));
        })
    };

    let on_season_focus = {
        let dispatch = dispatch.clone();
        Callback::from(move |_| dispatch.dispatch(DateAction::SeasonFocus(true)))
    };
    let on_season_blur = {
        let dispatch = dispatch.clone();
        Callback::from(move |_| dispatch.dispatch(DateAction::SeasonFocus(false)))
    };

    let make_season_select = {
        let dispatch = dispatch.clone();
        move |season: u8| {
            let dispatch = dispatch.clone();
            Callback::from(move |_| dispatch.dispatch(DateAction::SeasonSelect(season)))
        }
    };

    let on_year_input = {
        let dispatch = dispatch.clone();
        Callback::from(move |e: InputEvent| {
            let value = e.target_unchecked_into::<HtmlInputElement>().value();
            dispatch.dispatch(DateAction::YearInput(value));
        })
    };

    let season_label = match state.season {
        Some(1) => "Spring",
        Some(2) => "Summer",
        Some(3) => "Fall",
        Some(4) => "Winter",
        _ => "Season",
    };

    let day_display = state.day.map(|d| d.to_string()).unwrap_or_default();
    let year_display = state.year.map(|y| y.to_string()).unwrap_or_default();

    html! {
        <div class="field has-addons">
            <div class="control" style="flex: 1">
                <input class="input" type="text" placeholder="Day" value={day_display} oninput={on_day_input} />
            </div>
            <div class={ if state.season_focus { "dropdown is-active" } else { "dropdown" } } style="flex: 1">
                <div class="control is-expanded">
                    <div class="dropdown-trigger">
                        <button class="button is-fullwidth is-justify-content-space-between" onfocus={on_season_focus} onblur={on_season_blur}>
                            <span>{ season_label }</span>
                            <span class="material-symbols-outlined">{ "expand_more" }</span>
                        </button>
                    </div>
                </div>
                <div class="dropdown-menu" style="width:100%">
                    <div class="dropdown-content">
                        <a class="dropdown-item" onmousedown={make_season_select(1)}>{ "Spring" }</a>
                        <a class="dropdown-item" onmousedown={make_season_select(2)}>{ "Summer" }</a>
                        <a class="dropdown-item" onmousedown={make_season_select(3)}>{ "Fall" }</a>
                        <a class="dropdown-item" onmousedown={make_season_select(4)}>{ "Winter" }</a>
                    </div>
                </div>
            </div>
            <div class="control" style="flex: 1">
                <input class="input" type="text" placeholder="Year" value={year_display} oninput={on_year_input} />
            </div>
        </div>
    }
}
