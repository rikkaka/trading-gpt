#![allow(non_snake_case)]
use dioxus::prelude::*;

#[derive(PartialEq, Props)]
pub struct ContentProps {
    content: String,
}

pub fn UserMessage(cx: Scope<ContentProps>) -> Element {
    cx.render(rsx!(
        div {
            class: "chat-message user-message",
            for line in cx.props.content.lines() {
                "{line}"
                br {}
            }
        }
    ))
}

pub fn OtherMessage(cx: Scope<ContentProps>) -> Element {
    cx.render(rsx!(
        div {
            class: "chat-message other-message",
            for line in cx.props.content.lines() {
                "{line}"
                br {}
            }
        }
    ))
}

pub fn Loading(cx: Scope) -> Element {
    cx.render(rsx!(
        div {
            class: "chat-message other-message",
            div {
                class: "spinner",
            }
        }
    ))
}

#[derive(Props)]
pub struct DraftProps<'a> {
    draft: &'a UseRef<String>,
    clean: &'a UseState<bool>,
    on_press: EventHandler<'a, Event<KeyboardData>>,
}

pub fn UserInput<'a>(cx: Scope<'a, DraftProps>) -> Element<'a> {
    let draft = cx.props.draft;
    let clean = cx.props.clean;

    if **clean {
        clean.set(false);
        cx.render(rsx!(textarea {
            id: "user-input",
            placeholder: "Type your message here",
            value: "",
            oninput: |e| {
                draft.set(e.value.clone());
            },
        }))
    } else {
        cx.render(rsx!(textarea {
            id: "user-input",
            placeholder: "Type your message here",
            oninput: |e| {
                draft.set(e.value.clone());
            },
            onkeypress: |e| cx.props.on_press.call(e),
        }))
    }
}
