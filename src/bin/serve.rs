use log::{info, debug};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() {
    std::env::set_var("RUST_LOG", "debug");
    // tracing_subscriber::registry()
    //     .with(fmt::layer())
    //     .with(EnvFilter::from_default_env())
    //     .init();

    pretty_env_logger::init();
    debug!("Hello, world!");
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(trading_gpt::start_server())
}
