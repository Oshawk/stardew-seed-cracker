use yew::prelude::*;

use crate::traveling_merchant::Platform;

pub enum PlatformMessage {
    PlatformFocus(bool),
    PlatformValue(Platform),
}

#[derive(Clone, PartialEq, Properties)]
pub struct PlatformProperties {
    pub callback: Callback<Option<Platform>>,
}

pub struct PlatformComponent {
    platform_focus: bool,
    platform_value: Option<Platform>,
}

impl Component for PlatformComponent {
    type Message = PlatformMessage;
    type Properties = PlatformProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            platform_focus: false,
            platform_value: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Self::Message::PlatformFocus(focus) => {
                self.platform_focus = focus;
            }
            Self::Message::PlatformValue(platform) => {
                self.platform_value = Some(platform);
            }
        }

        ctx.props().callback.emit(self.platform_value);

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class="field has-addons">
                <div class={ if self.platform_focus { "dropdown is-active" } else { "dropdown" } } style="flex: 1">
                    <div class="control is-expanded">
                        <div class="dropdown-trigger">
                            <button class="button is-fullwidth is-justify-content-space-between" onfocus={ ctx.link().callback(|_| Self::Message::PlatformFocus(true)) } onblur={ ctx.link().callback(|_| Self::Message::PlatformFocus(false)) }>
                                <span>{ match self.platform_value { Some(value) => match value { Platform::PC => "PC".to_string(), Platform::Switch => "Switch".to_string() }, None => "Platform".to_string() } }</span>
                                <span class="material-symbols-outlined">{ "expand_more" }</span>
                            </button>
                        </div>
                    </div>
                    <div class="dropdown-menu" style="width:100%">
                        <div class="dropdown-content">
                            <a class="dropdown-item" onmousedown={ ctx.link().callback(move |_| Self::Message::PlatformValue(Platform::PC)) }>{ "PC" }</a>
                            <a class="dropdown-item" onmousedown={ ctx.link().callback(move |_| Self::Message::PlatformValue(Platform::Switch)) }>{ "Switch" }</a>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}
