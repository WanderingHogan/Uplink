use std::fs;

use dioxus::prelude::*;
use dioxus_heroicons::outline::Shape;
use warp::crypto::DID;

use ui_kit::{
    button::{Button, State},
    extension_placeholder::ExtensionPlaceholder,
    switch::Switch,
};

use crate::iutils::config::Config;
use ::utils::Account;

#[derive(Props, PartialEq)]
pub struct Props {
    account: Account,
}

#[allow(non_snake_case)]
pub fn Developer(cx: Scope<Props>) -> Element {
    log::debug!("rendering settings/pages/Developer");
    let mut config = Config::load_config_or_default();
    let c = config.clone();

    let did = if let Ok(ident) = cx.props.account.get_own_identity() {
        ident.did_key().to_string()
    } else {
        DID::default().to_string()
    };

    let cache_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".warp")
        .into_os_string()
        .into_string()
        .unwrap_or_default();

    cx.render(rsx! {
        div {
            id: "page_developer",
            class: "padded",
            div {
                class: "item",
                    div {
                    class: "description",
                    label {
                        "Developer Mode"
                    },
                    p {
                        "Enabling developer mode adds logging and displays helpful debug information on the UI."
                    }
                },
                div {
                    class: "interactive",
                    Switch {
                        active: config.developer.developer_mode,
                        on_change: move |_| {
                            config.developer.developer_mode = !config.developer.developer_mode;
                            let _ = config.save();
                        }
                    }
                }
            }
            div {
                class: "item",
                div {
                    class: "description",
                    label {
                        "Open Cache"
                    },
                    p {
                        "Open the cache in your default file browser."
                    }
                },
                div {
                    class: "interactive",
                    Button {
                        icon: Shape::FolderOpen,
                        disabled: false,
                        text: String::from("Open Cache"),
                        on_pressed: move |_| {
                            let _ = opener::open(&cache_path);
                        },
                    }
                }
            }
            div {
                class: "item",
                div {
                    class: "description",
                    label {
                        "Extract Cache"
                    },
                    p {
                        "Zips and downloads your cache folder for sharing with other developers or migration to another device."
                    }
                },
                div {
                    class: "interactive",
                    Button {
                        icon: Shape::ArrowDownOnSquare,
                        disabled: true,
                        text: String::from("Download"),
                        on_pressed: move |_| {},
                    }
                }
            }
            div {
                class: "item",
                div {
                    class: "description",
                    label {
                        "Reset Cache"
                    },
                    p {
                        "Removes your cache and reloads the app, this is useful for testing new accounts quickly."
                    }
                },
                div {
                    class: "interactive",
                    Button {
                        icon: Shape::Trash,
                        state: State::Secondary,
                        text: String::from("Reset"),
                        on_pressed: move |_| {
                            let c = dirs::home_dir()
                                .unwrap_or_default()
                                .join(".warp")
                                .into_os_string()
                                .into_string()
                                .unwrap_or_default();
                            let _ = fs::remove_dir_all(c);
                        },
                    }
                }
            }
            div {
                class: "item",
                div {
                    class: "description",
                    label {
                        "DIDKey"
                    },
                    p {
                        class: "selectable",
                        "{did}"
                    }
                },
                div {
                    class: "interactive",
                }
            }
            (c.developer.developer_mode).then(|| rsx! {
                ExtensionPlaceholder {},
            })
        }
    })
}
