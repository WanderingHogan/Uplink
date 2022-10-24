use dioxus::prelude::*;
use dioxus_heroicons::{Icon, solid::Shape};

#[derive(Props, PartialEq)]
pub struct Props {
    account: crate::Account,
}

#[allow(non_snake_case)]
pub fn Sidebar(cx: Scope<Props>) -> Element {
    cx.render(rsx! {
        // TODO: This should be generated based on actual content later.
        // We will create reusable components for a folder and just pass children as data and render
        // recursively automatically.
        // This is just to work on css
        div {
            id: "sidebar",
            class: "tree noselect",
            div {
                class: "tree_wrapper",
                label {
                    class: "tree_folder root",
                    div {
                        class: "row",
                        Icon {
                            icon: Shape::Folder,
                        },
                        "Folder 1"
                    },
                    input {
                        id: "tree_folder_control",
                        "type": "checkbox",
                    },
                    label {
                        class: "tree_folder",
                        div {
                            class: "row",
                            Icon {
                                icon: Shape::Folder,
                            },
                            "SubFolder 1"
                        },
                        input {
                            id: "tree_folder_control",
                            "type": "checkbox",
                        },
                        a {
                            class: "tree_item",
                            div {
                                class: "row",
                                Icon {
                                    icon: Shape::Document,
                                },
                                "Item"
                            },
                        },
                    },
                    label {
                        class: "tree_folder",
                        div {
                            class: "row",
                            Icon {
                                icon: Shape::Folder,
                            },
                            "Subfolder 2",
                        },
                        input {
                            id: "tree_folder_control",
                            "type": "checkbox",
                        },
                        label {
                            class: "tree_folder",
                            div {
                                class: "row",
                                Icon {
                                    icon: Shape::Folder,
                                },
                                "Subfolder 1",
                            },
                            input {
                                id: "tree_folder_control",
                                "type": "checkbox",
                            },
                            label {
                                class: "tree_folder",
                                div {
                                    class: "row",
                                    Icon {
                                        icon: Shape::Folder,
                                    },
                                    "Subfolder 2",
                                },
                                input {
                                    id: "tree_folder_control",
                                    "type": "checkbox",
                                },
                                a {
                                    class: "tree_item",
                                    div {
                                        class: "row",
                                        Icon {
                                            icon: Shape::Document,
                                        },
                                        "Item"
                                    },
                                },
                            }
                            a {
                                class: "tree_item",
                                div {
                                    class: "row",
                                    Icon {
                                        icon: Shape::Document,
                                    },
                                    "Item"
                                },
                            },
                        }
                        a {
                            class: "tree_item",
                            div {
                                class: "row",
                                Icon {
                                    icon: Shape::Document,
                                },
                                "Item"
                            },
                        },
                    }
                },
                label {
                    class: "tree_folder root",
                    div {
                        class: "row",
                        Icon {
                            icon: Shape::Folder,
                        },
                        "Folder 2"
                    }
                    input {
                        id: "tree_folder_control",
                        "type": "checkbox",
                    },
                    a {
                        class: "tree_item",
                        div {
                            class: "row",
                            Icon {
                                icon: Shape::Document,
                            },
                            "Item"
                        },
                    },
                },
            }
        }
    })
}