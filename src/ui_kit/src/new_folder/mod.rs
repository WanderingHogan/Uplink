use dioxus::prelude::*;
use dioxus_heroicons::{outline::Shape, Icon};
use dioxus_html::KeyCode;

use super::folder::State;

// Remember: owned props must implement PartialEq!
#[derive(PartialEq, Eq, Props)]
pub struct Props {
    state: State,
}

#[allow(non_snake_case)]
pub fn NewFolder(cx: Scope<Props>) -> Element {
    let class = match cx.props.state {
        State::Primary => "primary",
        State::Secondary => "secondary",
    };

    let folder_name = use_state(&cx, || String::from("New Folder"));

    cx.render(rsx! {
        div {
            class: "folder {class}",
            Icon { icon: Shape::Folder },
            input {
                class: "new_folder_input",
                autofocus: "true",
                placeholder: "New Folder",
                oninput: move |evt| {
                    folder_name.set(evt.value.to_string());
                },
                onkeyup: |evt| {
                    if evt.key_code == KeyCode::Enter {
                        println!("Create new folder: {}", folder_name.clone());
                    }
                }
            }
        }
    })
}
