use axum::{
    extract::ws::WebSocketUpgrade,
    response::Html,
    routing::get,
    Router,
};
use dioxus::prelude::*;

use crate::trading_core::Bot;
use super::types::{Message, Role};

fn app(cx: Scope) -> Element {
    let bot = use_ref(cx, || Bot::new());
    let draft = use_ref(cx, || String::new());
    let messages = use_ref(cx, || Vec::<Message>::new());
    let send_lock = use_state(cx, || false);

    let send = move |_| {
        if send_lock == true {return;}
        let tmp = draft.read().clone();
        if tmp.len() == 0 {return;}
        messages.write().push(Message {
            role: Role::User,
            content: draft.read().clone(),
        });
        messages.write().push(Message {
            role: Role::Bot,
            content: "Please wait...".into(),
        });
        draft.set(String::new());

        cx.spawn({
            send_lock.set(true);
            let send_lock = send_lock.to_owned();
            let bot = bot.to_owned();
            let messages = messages.to_owned();
            
            async move {
                // sleep 2 seconds
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                let response = bot.write().chat(&tmp).await;
                messages.write().last_mut().unwrap().update_content(response);
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
                div {
                    class: match msg.role {
                        Role::User => "chat-message user-message",
                        Role::Bot => "chat-message other-message",
                    },
                    "{msg.content}"
                }
            }
        }
        div {
            id: "input-area",
            textarea {
                id: "user-input",
                value: "{draft.read()}",
                oninput: |evt| draft.set(evt.value.clone()),
            }
            button { 
                id: "send-button",
                onclick: send, "发送" }
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
