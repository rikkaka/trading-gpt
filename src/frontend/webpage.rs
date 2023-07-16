use axum::{extract::ws::WebSocketUpgrade, response::Html, routing::get, Router};
use dioxus::prelude::*;

use crate::trading_core::Bot;

struct Message {
    role: Role,
    content: String,
}

enum Role {
    User,
    Bot,
}

fn app(cx: Scope) -> Element {
    let draft = use_state(cx, || String::new());
    let bot = use_ref(cx, || Bot::new());
    let messages = use_ref(cx, || Vec::<Message>::new());

    cx.render(rsx!(
        style { include_str!("./style.css") }
        div {
            id: "chat-window",
            class: "chat-window",
            messages.read().iter().map(|msg| {
                rsx!(div {
                    class: match msg.role {
                        Role::User => "chat-message user-message",
                        Role::Bot => "chat-message bot-message",
                    },
                    "{msg.content}"
                })
            })
        }
        div {
            id: "input-area",
            textarea {
                cols: 50,
                id: "user-input",
                value: "{draft}",
                oninput: |evt| draft.set(evt.value.clone()),

            }
            button { onclick: move |_| {
                let mut bot = bot.write();
                let mut messages = messages.write();
                messages.push(Message {
                    role: Role::User,
                    content: (*draft.current()).clone(),
                });
                let tmp = draft.clone();
                draft.set(String::new());
                let response = bot.chat(&tmp);
                messages.push(Message {
                    role: Role::Bot,
                    content: response,
                });
            }, "发送" }
        }
    ))
}

pub async fn start_server() {
    let addr: std::net::SocketAddr = ([127, 0, 0, 1], 3030).into();

    let view = dioxus_liveview::LiveViewPool::new();

    let app = Router::new()
        .route(
            "/",
            get(move || async move {
                Html(format!(
                    r#"
            <!DOCTYPE html>
            <html>
                <head> <title>Trading GPT</title>  </head>
                <body> <div id="main"></div> </body>
                {glue}
            </html>
            "#,
                    glue = dioxus_liveview::interpreter_glue(&format!("ws://{addr}/ws"))
                ))
            }),
        )
        .route(
            "/ws",
            get(move |ws: WebSocketUpgrade| async move {
                ws.on_upgrade(move |socket| async move {
                    _ = view.launch(dioxus_liveview::axum_socket(socket), app).await;
                })
            }),
        );

    println!("Listening on http://{addr}");

    axum::Server::bind(&addr.to_string().parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
