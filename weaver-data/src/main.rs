//! Command line tool to examine the weaver data.
//!
//! # Installation
//!
//! When first installing weaver you need to setup your stores by running
//!
//! `weaver-data setup`
//!
//! # Development
//!
//! During development you can setup a staging environment with the following command
//!
//! `RUST_BACKTRACE=1 WEAVER=debug cargo run -p weaver-data -- -C weaver-staging  -P setup`
//!
//! The [weaver-server](../weaver_server/index.html) needs to be setup accordingly.
//!

mod app;
mod cli;

fn main() {
    // Setup the logger on the env variable WEAVER.
    // This allows one to do `export WEAVER=debug` to get a lot more errors.
    use env_logger::Builder;
    use std::env;

    // Use the builder api for more flexibility.
    let mut builder = Builder::new();
    if env::var("WEAVER").is_ok() {
        builder.parse(&env::var("WEAVER").unwrap());
    }
    builder.init();

    // Run the main loop, be concise with error reporting since we may run in PS1.
    if let Err(ref e) = app::run() {
        println!(" ERR `export WEAVER=error` for more ");
        println!("error {}", e);
        ::log::error!("error {}", e);

        e.display();

        ::std::process::exit(1);
    }
    ::log::info!("weaver-data exited normally");
}
