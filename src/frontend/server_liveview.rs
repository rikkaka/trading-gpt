use axum::{extract::ws::WebSocketUpgrade, response::Html, routing::get, Router};
use dotenvy::dotenv;

use super::app::app;

pub async fn start_server() {
    dotenv().ok();
    let reachable_addr = std::env::var("REACHABLE_ADDR").unwrap();
    let listen_addr = std::env::var("LISTEN_ADDR").unwrap();

    let addr: std::net::SocketAddr = listen_addr.parse().unwrap();

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
