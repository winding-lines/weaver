//! Shell cli interface. To execute from the build environment run with
//!
//! `cargo run -p cli-shell -- --help`
//!
//! The shell process manages the current epic in a local json store and reads the rest
//! of the information from the [weaver-server](../../cli_server/index.html).
//!
extern crate chan;
extern crate chrono;
extern crate clap;
extern crate clipboard;
extern crate cursive;
extern crate cursive_table_view;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate termion;
extern crate walkdir;
// Workspace crates
extern crate weaver_error;
extern crate weaver_rpc;
extern crate lib_api;


mod cli;
mod display;
mod controllers;
mod local_api;
mod local_store;

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
    builder.init();

    // Run the main loop, be concise with error reporting since we may run in PS1.
    if let Err(ref e) = controllers::app::run() {
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
