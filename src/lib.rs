pub mod common;
mod frontend;
mod global;
mod trading_core;

pub use frontend::start_server;

pub fn foo() {
    print!("Hello world!")
}
