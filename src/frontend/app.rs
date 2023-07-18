use dioxus::prelude::*;
use log::debug;
use tokio::sync::mpsc;

use super::components::*;
use super::types::*;
use crate::trading_core::Bot;

pub fn app(cx: Scope) -> Element {
    let (tx, rx) = mpsc::channel::<String>(32);

    let bot = use_ref(cx, || Bot::new(tx));
    let draft = use_ref(cx, String::new);
    let messages = use_ref(cx, Vec::<Message>::new);
    let send_lock = use_state(cx, || false);
    let clean = use_state(cx, || false);
    let loading = use_state(cx, || false);

    use_future(cx, (), move |_| {
        let mut rx = rx;
        let messages = messages.to_owned();
        async move {
            while let Some(msg) = rx.recv().await {
                messages.write().push(Message {
                    role: Role::Bot,
                    content: msg,
                })
            }
        }
    });

    let send = move |_| {
        if send_lock == true {
            return;
        }
        let tmp = draft.read().clone();
        if tmp.is_empty() {
            return;
        }
        send_lock.set(true);
        loading.set(true);
        clean.set(true);
        messages.write().push(Message {
            role: Role::User,
            content: draft.read().replace('\n', "<br>"),
        });

        cx.spawn({
            let send_lock = send_lock.to_owned();
            let loading = loading.to_owned();
            let bot = bot.to_owned();
            let messages = messages.to_owned();

            async move {
                debug!("Sending message: {}", tmp);
                bot.write().chat(&tmp).await.unwrap_or_else(|err| {
                    messages.write().push(Message::new(
                        Role::Bot,
                        format!("Error: {}", err),
                    ));
                });

                loading.set(false);
                send_lock.set(false);
            }
        })
    };

    cx.render(rsx!(
        style { include_str!("./style.css") }
        div {
            id: "header",
            h1 {"A demo intilligent payment system"}
            h2 {"Powered by ChatGPT"}
        }
        div {
            id: "chat-window",
            class: "chat-window",
            for msg in messages.read().iter() {
                match msg.role {
                    Role::User => rsx!(UserMessage { content: msg.content.clone() }),
                    Role::Bot => rsx!(OtherMessage { content: msg.content.clone() }),
                }
            }
            if loading == true {
                rsx!(Loading{})
            }
        }
        div {
            id: "input-area",
            UserInput {
                draft: draft,
                clean: clean
            }
            button {
                id: "send-button",
                onclick: send, "Send" }
        }
        div {
            id: "bottom-holder"
        }
    ))
}