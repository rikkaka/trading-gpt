use axum::{extract::ws::WebSocketUpgrade, response::Html, routing::get, Router};
use dioxus::prelude::*;
use log::debug;
use tokio::sync::mpsc;

use super::components::*;
use super::types::*;
use crate::trading_core::Bot;
use dotenvy::dotenv;

fn app(cx: Scope) -> Element {
    let (tx, rx) = mpsc::channel::<String>(32);

    let bot = use_ref(cx, || Bot::new(tx));
    let draft = use_ref(cx, || String::new());
    let messages = use_ref(cx, || Vec::<Message>::new());
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
        send_lock.set(true);
        loading.set(true);
        clean.set(true);

        let tmp = draft.read().clone();
        if tmp.len() == 0 {
            return;
        }
        messages.write().push(Message {
            role: Role::User,
            content: draft.read().replace("\n", "<br>").clone(),
        });
        // messages
        //     .write()
        //     .push(Message::new(Role::Loading, "Please wait".into()));
        // draft.set(String::new());

        cx.spawn({
            let send_lock = send_lock.to_owned();
            let loading = loading.to_owned();
            let bot = bot.to_owned();
            let messages = messages.to_owned();

            async move {
                bot.write().chat(&tmp).await.unwrap_or_else(|err| {
                    messages.write().push(Message::new(
                        Role::Bot,
                        format!("Error: {}", err.to_string()),
                    ));
                });

                // let response = bot.write().chat(&tmp).await;
                // messages.write().last_mut().unwrap().loaded(response.replace("\n","<br>"));
                loading.set(false);
                send_lock.set(false);
            }
        })
    };

    cx.render(rsx!(
        style { include_str!("./style.css") }
        div {
            id: "header",
            h1 {"An intilligent payment system"}
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

pub async fn start_server() {
    dotenv().ok();
    let reachable_addr = std::env::var("REACHABLE_ADDR").unwrap();
    let port = std::env::var("PORT").unwrap().parse::<u16>().unwrap();

    let addr: std::net::SocketAddr = ([0, 0, 0, 0], port).into();

    let view = dioxus_liveview::LiveViewPool::new();

    let app = Router::new()
        .route(
            "/",
            get(move || async move {
                Html(format!(
                    r#"
            <!DOCTYPE html>
            <html>
                <head> 
                    <title>Trading GPT</title>  
                    <meta name="viewport" 
                    content="width=device-width, 
                    initial-scale=1, 
                    minimum-scale=1,
                    maximum-scale=1,
                    user-scalable=no">
                </head>
                <body> <div id="main"></div> </body>
                {glue}
            </html>
            "#,
                    glue = dioxus_liveview::interpreter_glue(&format!("ws://{reachable_addr}/ws"))
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
