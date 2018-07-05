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
extern crate bincode;
extern crate chrono;
extern crate clap;
extern crate env_logger;
extern crate lib_api;
extern crate lib_db;
extern crate lib_error;
extern crate lib_index;
#[macro_use]
extern crate log;
extern crate serde;
extern crate serde_json;


mod cli;
mod app;

fn main() {
    // Setup the logger on the env variable WEAVER.
    // This allows one to do `export WEAVER=debug` to get a lot more errors.
    use std::env;
    use env_logger::{Builder, Target};

    // Use the builder api for more flexibility.
    let mut builder = Builder::new();
    // send output to stderr in order to be able to debug Cursive layer
    builder.target(Target::Stderr);
    if env::var("WEAVER").is_ok() {
        builder.parse(&env::var("WEAVER").unwrap());
    }
    builder.filter_module("actix_web", log::LevelFilter::Info);
    builder.filter_module("tantivy", log::LevelFilter::Info);
    builder.init();

    // Run the main loop, be concise with error reporting since we may run in PS1.
    if let Err(ref e) = app::run() {
        print!(" ERR `export WEAVER=error` for more");
        error!("error {}", e);

        for e in e.iter().skip(1) {
            error!("caused by: {}", e);
        }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            error!("backtrace: {:?}", backtrace);
        }
        error!(" `export WEAVER=debug` for more details.");

        ::std::process::exit(1);
    }
    info!("weaver exited normally");
}
