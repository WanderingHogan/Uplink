use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use crate::{
    components::main::compose::{divider::Divider, msg::Msg, reply::Reply},
    iutils,
    state::{Actions, LastMsgSent},
    Account, Messaging, STATE,
};
use dioxus::prelude::*;
use dioxus_heroicons::{outline::Shape, Icon};

use futures::StreamExt;
use uuid::Uuid;
use warp::{
    crypto::DID,
    raygun::{Message, MessageEvent, MessageEventKind, MessageOptions},
};

#[derive(Eq, PartialEq)]
enum TypingIndicator {
    Typing,
    NotTyping,
}

#[allow(clippy::large_enum_variant)]
enum ChanCmd {
    Indicator {
        users_typing: UseRef<HashMap<DID, String>>,
        current_chat: Option<Uuid>,
        remote_id: DID,
        remote_name: String,
        indicator: TypingIndicator,
    },
    Timeout {
        users_typing: UseRef<HashMap<DID, String>>,
        current_chat: Option<Uuid>,
    },
}

#[derive(Props, PartialEq)]
pub struct Props {
    account: Account,
    messaging: Messaging,
    users_typing: UseRef<HashMap<DID, String>>,
}

#[allow(non_snake_case)]
pub fn Messages(cx: Scope<Props>) -> Element {
    log::debug!("rendering Messages");

    //Note: We will just unwrap for now though we need to
    //      handle the error properly if there is ever one when
    //      getting own identity
    let state = use_atom_ref(&cx, STATE).clone();

    let mut rg = cx.props.messaging.clone();
    let ident = cx.props.account.get_own_identity().unwrap();
    let my_did = ident.did_key();
    // this one has a special name because of the other variable names within the use_future
    let list: UseRef<Vec<Message>> = use_ref(&cx, Vec::new).clone();
    // this one is for the rsx! macro. it is reversed for display purposes and defined here because `list` gets moved into the use_future
    let messages: Vec<Message> = list.read().iter().cloned().collect();

    // this is used for reading the event stream.
    let current_chat = state
        .read()
        .selected_chat
        .and_then(|x| state.read().active_chats.get(&x).cloned());

    let first_unread_message_id = current_chat
        .clone()
        .unwrap_or_default()
        .first_unread_message_id
        .unwrap_or_default();

    let msg_script = include_str!("messages.js");

    // periodically refresh message timestamps
    use_future(&cx, (), move |_| {
        let update = cx.schedule_update();
        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                update();
            }
        }
    });

    // keep track of who is typing by receiving events and adding timeouts
    let chan = use_coroutine(&cx, |mut rx: UnboundedReceiver<ChanCmd>| async move {
        // used for timeouts
        let mut typing_times: HashMap<DID, Instant> = HashMap::new();
        let mut prev_current_chat: Option<Uuid> = None;

        while let Some(cmd) = rx.next().await {
            match cmd {
                ChanCmd::Indicator {
                    users_typing,
                    current_chat,
                    remote_id,
                    remote_name,
                    indicator,
                } => {
                    //log::debug!("received typing indicator");
                    if current_chat != prev_current_chat {
                        typing_times.clear();
                        prev_current_chat = current_chat;
                    }
                    if current_chat.is_some() {
                        match indicator {
                            TypingIndicator::Typing => {
                                typing_times.insert(remote_id.clone(), Instant::now());
                                if !users_typing.read().contains_key(&remote_id) {
                                    users_typing.write().insert(remote_id, remote_name);
                                }
                            }
                            TypingIndicator::NotTyping => {
                                typing_times.remove(&remote_id);
                                if users_typing.read().contains_key(&remote_id) {
                                    let _ = users_typing.write().remove(&remote_id);
                                }
                            }
                        }
                    }
                }
                ChanCmd::Timeout {
                    users_typing,
                    current_chat,
                } => {
                    //log::debug!("received typing indicator timeout");
                    if current_chat != prev_current_chat {
                        typing_times.clear();
                        prev_current_chat = current_chat;
                    }
                    if current_chat.is_some() {
                        let expired_indicators: HashMap<DID, Instant> = typing_times
                            .iter()
                            .filter(|(_k, v)| {
                                let elapsed = Instant::now().duration_since(**v);
                                elapsed > Duration::from_secs(3)
                            })
                            .map(|(k, v)| (k.clone(), *v))
                            .collect();

                        let new_users_typing: HashMap<DID, String> = users_typing
                            .read()
                            .iter()
                            .filter(|(k, _v)| !expired_indicators.contains_key(k))
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect();

                        for (k, _v) in expired_indicators {
                            let _ = typing_times.remove(&k);
                        }

                        if new_users_typing != *users_typing.read() {
                            *users_typing.write() = new_users_typing;
                        }
                    }
                }
            }
        }
    });

    // periodically check for timeouts
    let chan1 = chan.clone();
    let real_current_chat = state.read().selected_chat;
    use_future(
        &cx,
        (&real_current_chat.clone(), &cx.props.users_typing.clone()),
        |(current_chat, users_typing)| async move {
            loop {
                //log::debug!("checking for typing indicator timeout on rx side");
                tokio::time::sleep(Duration::from_secs(4)).await;
                chan1.send(ChanCmd::Timeout {
                    users_typing: users_typing.clone(),
                    current_chat,
                });
            }
        },
    );

    // handle message stream
    let chan2 = chan.clone();
    use_future(
        &cx,
        (
            &current_chat,
            &cx.props.users_typing.clone(),
            &cx.props.account.clone(),
        ),
        |(current_chat, users_typing, mp)| async move {
            // don't stream messages from a nonexistent conversation
            let mut current_chat = match current_chat {
                // this better not panic
                Some(c) => c,
                None => return,
            };

            if current_chat.num_unread_messages != 0 {
                current_chat.num_unread_messages = 0;
                state
                    .write_silent()
                    .dispatch(Actions::UpdateConversation(current_chat.clone()));
            }

            let mut stream = loop {
                match rg
                    .get_conversation_stream(current_chat.conversation.id())
                    .await
                {
                    Ok(stream) => break stream,
                    Err(e) => match &e {
                        warp::error::Error::RayGunExtensionUnavailable => {
                            //Give sometime for everything in the background to fully line up
                            //Note, if this error still happens, it means there is an fatal error
                            //      in the background
                            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                        }
                        _ => {
                            // todo: properly report this error
                            // eprintln!("failed to get_conversation_stream: {}", e);
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        }
                    },
                }
            };

            let messages = rg
                .get_messages(current_chat.conversation.id(), MessageOptions::default())
                .await
                .unwrap_or_default();

            //This is to prevent the future updating the state and causing a rerender
            if *list.read() != messages {
                log::debug!("updating messages list ");
                *list.write() = messages;
            }

            while let Some(event) = stream.next().await {
                match event {
                    MessageEventKind::MessageReceived {
                        conversation_id,
                        message_id,
                    }
                    | MessageEventKind::MessageSent {
                        conversation_id,
                        message_id,
                    } => {
                        if current_chat.conversation.id() == conversation_id {
                            match rg.get_message(conversation_id, message_id).await {
                                Ok(message) => {
                                    log::debug!("compose/messages streamed a new message ");
                                    // remove typing indicator
                                    let username =
                                        iutils::get_username_from_did(message.sender(), &mp);
                                    chan2.send(ChanCmd::Indicator {
                                        users_typing: users_typing.clone(),
                                        current_chat: Some(conversation_id),
                                        remote_id: message.sender(),
                                        remote_name: username,
                                        indicator: TypingIndicator::NotTyping,
                                    });
                                    // update messages
                                    // todo: check if sidebar gets updated
                                    list.write().push(message.clone());
                                    current_chat.last_msg_sent =
                                        Some(LastMsgSent::new(&message.value()));
                                    state.write().dispatch(Actions::UpdateConversation(
                                        current_chat.clone(),
                                    ));
                                }
                                Err(_e) => {
                                    // todo: log error
                                }
                            }
                        }
                    }
                    MessageEventKind::EventReceived {
                        conversation_id,
                        did_key,
                        event,
                    } => match event {
                        MessageEvent::Typing => {
                            if current_chat.conversation.id() == conversation_id
                                && did_key != my_did
                            {
                                let username = iutils::get_username_from_did(did_key.clone(), &mp);
                                chan2.send(ChanCmd::Indicator {
                                    users_typing: users_typing.clone(),
                                    current_chat: Some(conversation_id),
                                    remote_id: did_key,
                                    remote_name: username,
                                    indicator: TypingIndicator::Typing,
                                })
                            }
                        }
                    },
                    MessageEventKind::EventCancelled {
                        conversation_id,
                        did_key,
                        event,
                    } => match event {
                        // this event isn't expected to be sent. handling it here anyway.
                        MessageEvent::Typing => {
                            if current_chat.conversation.id() == conversation_id
                                && did_key != my_did
                            {
                                let username = iutils::get_username_from_did(did_key.clone(), &mp);
                                chan2.send(ChanCmd::Indicator {
                                    users_typing: users_typing.clone(),
                                    current_chat: Some(conversation_id),
                                    remote_id: did_key,
                                    remote_name: username,
                                    indicator: TypingIndicator::NotTyping,
                                })
                            }
                        }
                    },
                    _ => {}
                }
            }
        },
    );

    let rg = cx.props.messaging.clone();
    let senders: Vec<DID> = current_chat
        .map(|info| info.conversation.recipients())
        .unwrap_or_default();
    let messages_len = messages.len();

    // get profile pictures for all senders in the conversation and cache them
    let mut profile_pictures = HashMap::new();
    for sender in senders.iter() {
        if profile_pictures.contains_key(&sender) {
            continue;
        }

        let profile_picture = iutils::get_pfp_from_did(sender.clone(), &cx.props.account.clone());
        profile_pictures.insert(sender, profile_picture);
    }

    cx.render(rsx! {
        div {
            id: "scroll-messages",
            class: "messages",
            div {
                class: "encrypted-notif",
                Icon {
                    icon: Shape::LockClosed
                }
                p {
                    "Messages secured by local E2E encryption."
                }
            },
            messages.iter()
                .enumerate()
                .map(|(idx, message)| {
                    let message_id = message.id();
                    let conversation_id = message.conversation_id();
                    let msg_sender = message.sender();
                    let is_remote = ident.did_key() != msg_sender;
                    let mut rg = rg.clone();
                    let sender_picture = profile_pictures.get(&msg_sender).and_then(|pbp| pbp.clone()).unwrap_or_default();

                    let is_first = if idx == 0 {
                        false
                    } else {
                        let prev_message = &messages[idx - 1];
                        prev_message.sender() != msg_sender
                    };

                    let is_last = if idx == messages.len() - 1 {
                        true
                    } else {
                        let next_message = &messages[idx + 1];
                        next_message.sender() != msg_sender
                    };

                    rsx! {
                        div {
                            key: "{message_id}",
                            style: "display: contents",
                            "data-remote": "{is_remote}",
                            message.replied().map(|replied| {
                                let r = cx.props.messaging.clone();
                                match warp::async_block_in_place_uncheck(r.get_message(conversation_id, replied)) {
                                    Ok(message) => {
                                        rsx!{
                                            Reply {
                                                // key: "{message_id}-reply",
                                                message_id: message.id(),
                                                message: message.value().join("\n"),
                                                attachments_len: message.attachments().len(),
                                                is_remote: is_remote,
                                                account: cx.props.account.clone(),
                                                sender: message.sender(),
                                            }
                                        }
                                    },
                                    // todo: if we don't want to display this, change message.replied().map to message.replied.and_then(), then 
                                    // in the match statement return Some(Element) on Ok and None on error, with error logging as desired. 
                                    Err(_) => { rsx!{ span { "Something went wrong" } } }
                                }
                            }),
                            (message_id == first_unread_message_id).then(||
                                rsx! {
                                    Divider {
                                        date: message.date(),
                                        num_unread: (messages_len - idx).try_into().unwrap(),
                                    }
                                }
                            )
                            Msg {
                                messaging: cx.props.messaging.clone(),
                                message: message.clone(),
                                account: cx.props.account.clone(),
                                sender: msg_sender,
                                remote: is_remote,
                                // not sure why this works. I believe the calculations for is_last and is_first are correct but for an unknown reason the time and profile picture gets displayed backwards.
                                last:  is_last,
                                first: is_first,
                                middle: !is_last && !is_first,
                                profile_picture: sender_picture,
                                on_reply: move |reply| {
                                    if let Err(_e) = warp::async_block_in_place_uncheck(rg.reply(conversation_id, message_id, vec![reply])) {
                                        //TODO: Display error?
                                    }
                                }
                            }
                        }
                    }
                }),
                script { "{msg_script}" 
            }
        }
    })
}
