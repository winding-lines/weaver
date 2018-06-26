//! Weaver is a history tracker for you command line and shell commands.
//! The long term goal is to combine with a recommender module that will use
//! the past actions to suggest the next one.
//!
//! Weaver relies on a couple of workspace crates.
//!
//! * [weaver-db](../weaver_db/index.html): holds the local database for actions
//! * [weaver-error](../weaver_error/index.html): error structs used by all crates
//! * [weaver-index](../weaver_index/index.html): full text search
//! * [weaver-rpc](../weaver_rpc/index.html): client implementation of API, used by the cli
//! * [weaver-web](../weaver_web/index.html): API and base page server

extern crate chan;
extern crate chrono;
extern crate clap;
extern crate clipboard;
extern crate cursive;
extern crate cursive_table_view;
extern crate daemonize;
extern crate env_logger;
extern crate futures;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate mime;
extern crate serde;
extern crate serde_json;
extern crate termion;
extern crate walkdir;
// Workspace crates
extern crate weaver_db;
extern crate weaver_error;
extern crate weaver_index;
extern crate weaver_rpc;
extern crate weaver_web;


mod cli;
mod display;
mod controllers;
mod local_api;

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
