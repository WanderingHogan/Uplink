use dioxus::prelude::*;
use sir::global_css;
use warp::raygun::Conversation;

use crate::{
    components::main::compose::{topbar::TopBar, write::Write},
    STATE, RAYGUN
};

#[derive(PartialEq, Props)]
pub struct Props {
    conversation: Conversation,
}

pub mod topbar;
pub mod write;
pub mod messages;

#[allow(non_snake_case)]
pub fn Compose(cx: Scope<Props>) -> Element {
    global_css!(
        "
        .compose {
            display: inline-flex;
            flex-direction: column;
            flex: 1;
            position: relative;

            .blurmask {
                -webkit-backdrop-filter: blur(3px);
                background: var(--theme-semi-transparent);
                position: absolute;
                top: 0;
                right: 0;
                bottom: 0;
                left: 0;
                z-index: 90;
            }
            .messages-container {
                flex: 1;
            }
            
            .writer-container {
                width: 100%;
                display: inline-flex;
            }
        }
    "
    );

    let state = use_atom_ref(&cx, STATE);
    let conversation_id = cx.props.conversation.id();

    // Load Multipass & Raygun's Atom Ref
    let raygun = use_atom_ref(&cx, RAYGUN);

    // Read their values from locks
    let rg = raygun.read().clone().unwrap().clone();

    let blur = state.read().chat.is_none();
    let text = use_state(&cx, || String::from(""));

    cx.render(rsx! {
        div {
            class: "compose",
            if blur {
                rsx!(
                    div {
                        class: "blurmask"
                    }
                )
            } else {
                rsx!(
                    TopBar {
                        conversation: cx.props.conversation.clone(),
                        on_call: move |_| {},
                    }
                )
            },
            div {
                class: "messages-container",
            },
            div {
                class: "writer-container",
                Write {
                    on_submit: move |message| {
                        text.set(String::from(""));
                        let send_message = use_future(&cx, (), |_| async move {
                            rg
                                .write()
                                .send_message()
                        });
                    },
                    on_upload: move |_| {}
                }
            }
        }
    })
}
