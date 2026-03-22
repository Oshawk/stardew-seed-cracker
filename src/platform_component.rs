use yew::prelude::*;

use crate::observation::Platform;

#[derive(Clone, PartialEq, Properties)]
pub struct PlatformProperties {
    pub callback: Callback<Option<Platform>>,
}

#[component]
pub fn PlatformComponent(props: &PlatformProperties) -> Html {
    let focus = use_state(|| false);
    let label = use_state(|| "Platform");

    let on_focus = {
        let focus = focus.clone();
        Callback::from(move |_| focus.set(true))
    };
    let on_blur = {
        let focus = focus.clone();
        Callback::from(move |_| focus.set(false))
    };

    let on_pc = {
        let label = label.clone();
        let callback = props.callback.clone();
        Callback::from(move |_| {
            label.set("PC");
            callback.emit(Some(Platform::PC));
        })
    };
    let on_switch = {
        let label = label.clone();
        let callback = props.callback.clone();
        Callback::from(move |_| {
            label.set("Switch");
            callback.emit(Some(Platform::Switch));
        })
    };

    html! {
        <div class="field has-addons">
            <div class={ if *focus { "dropdown is-active" } else { "dropdown" } } style="flex: 1">
                <div class="control is-expanded">
                    <div class="dropdown-trigger">
                        <button class="button is-fullwidth is-justify-content-space-between" onfocus={on_focus} onblur={on_blur}>
                            <span>{ *label }</span>
                            <span class="material-symbols-outlined">{ "expand_more" }</span>
                        </button>
                    </div>
                </div>
                <div class="dropdown-menu" style="width:100%">
                    <div class="dropdown-content">
                        <a class="dropdown-item" onmousedown={on_pc}>{ "PC" }</a>
                        <a class="dropdown-item" onmousedown={on_switch}>{ "Switch" }</a>
                    </div>
                </div>
            </div>
        </div>
    }
}
