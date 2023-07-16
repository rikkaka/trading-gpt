#![allow(non_snake_case)]

use dioxus::prelude::*;

#[derive(PartialEq, Props)]
pub struct ContentProps {
    content: String
}

pub fn UserMessage(cx: Scope<ContentProps>) -> Element {
    cx.render(rsx!(
        div {
            class: "chat-message user-message",
            "{cx.props.content}"
        }
    ))
}

pub fn OtherMessage(cx: Scope<ContentProps>) -> Element {
    cx.render(rsx!(
        div {
            class: "chat-message other-message",
            "{cx.props.content}"
        }
    ))
}

pub fn Loading(cx: Scope<ContentProps>) -> Element {
    cx.render(rsx!(
        div {
            class: "chat-message other-message",
            div {
                class: "spinner",
            }
        }
    ))
}