use crate::{
    components::reusable::textarea::TextArea, iutils::config::Config, Messaging, LANGUAGE,
};
use audio_factory::AudioFactory;
use dioxus::prelude::*;
use dioxus_heroicons::outline::Shape;
use incognito_typing::ExtIncognitoTyping;
use state::STATE;
use ui_kit::{
    button::{self, Button},
    context_menu::{ContextItem, ContextMenu},
    small_extension_placeholder::SmallExtensionPlaceholder,
};
use utils::extensions::{get_renders, BasicExtension, ExtensionType};

#[derive(Props)]
pub struct Props<'a> {
    messaging: Messaging,
    on_submit: EventHandler<'a, String>,
    on_upload: EventHandler<'a, ()>,
}

#[allow(non_snake_case)]
pub fn Write<'a>(cx: Scope<'a, Props<'a>>) -> Element<'a> {
    log::debug!("rendering compose/Write");
    let config = Config::load_config_or_default();

    let text = use_state(&cx, String::new);
    let l = use_atom_ref(&cx, LANGUAGE).read();
    let state = use_atom_ref(&cx, STATE).read();
    let ext_enabled = state.enabled_extensions.clone();

    let exts = get_renders(
        ExtensionType::ChatbarIcon,
        config.extensions.enable,
        ext_enabled.clone(),
    );

    cx.render(rsx! {
        div {
            class: "write",
            id: "write",
            ContextMenu {
                parent: String::from("write"),
                items: cx.render(rsx! {
                    ContextItem {
                        onpressed: move |_| {},
                        icon: Shape::Clipboard,
                        text: String::from("Copy Conversation ID")
                    },
                })
            },
            exts,
            Button {
                icon: Shape::Plus,
                on_pressed: move |_| {
                    let _ = &cx.props.on_upload.call(());
                },
            },
            TextArea {
                messaging: cx.props.messaging.clone(),
                on_input: move |_| {}
                on_submit: move |val| cx.props.on_submit.call(val),
                text: text.clone(),
                placeholder: l.chatbar_placeholder.to_string()
            }
            config.developer.developer_mode.then(|| rsx! {
                div {
                    class: "extension-holder",
                    SmallExtensionPlaceholder {}
                }
            })
            div {
                class: "chatbar_extensions",
                ext_enabled.clone().contains(&AudioFactory::info().name).then(|| rsx!{
                    AudioFactory::render()
                })
                ext_enabled.clone().contains(&ExtIncognitoTyping::info().name).then(|| rsx!{
                    ExtIncognitoTyping::render()
                })
            },
            div {
                id: "send",
                Button {
                    icon: Shape::ArrowRight,
                    state: button::State::Secondary,
                    on_pressed: move |_| {
                        let text = text.clone();
                        let _ = &cx.props.on_submit.call(text.to_string());
                        text.set(String::from(""));
                    },
                }
            }
        }
    })
}
