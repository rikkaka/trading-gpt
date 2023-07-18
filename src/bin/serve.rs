use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() {
    // std::env::set_var("RUST_LOG", "info");
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(trading_gpt::start_server())
}
