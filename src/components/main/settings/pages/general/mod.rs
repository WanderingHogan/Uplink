use dioxus::prelude::*;

use crate::{
    components::ui_kit::switch::Switch,
    utils::config::Config,
    Account,
};

#[derive(Props, PartialEq)]
pub struct Props {
    account: Account,
}


#[allow(non_snake_case)]
pub fn General(cx: Scope<Props>) -> Element {
    let mut config = Config::load_config_or_default();

    cx.render(rsx! {
        div {
            id: "page_general",
        div {
                class: "item",
        div {
                    class: "description",
        label {
                        "Splash Screen"
                    },
        p {
                        "Disabling the splash screen can sometimes make for a faster startup."
                    }
                },
        div {
                    class: "interactive",
        Switch {
                        active: config.general.show_splash,
        on_change: move |_| {
                            config.general.show_splash = !config.general.show_splash;
                            let _ = config.save();
                        }
                    }
                }
            }
        },
    })
}