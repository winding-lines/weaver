//! Weaver server, to execute from the build environment run with
//!
//! `cargo run -p cli-server -- check`
//!
//! The server provides a REST interface to the Weaver stores. The interface
//! is used by the shell and chrome extensions.
//!
extern crate chrono;
extern crate clap;
extern crate daemonize;
extern crate env_logger;
extern crate futures;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate mime;
extern crate serde;
extern crate serde_json;
extern crate lib_api;
extern crate weaver_db;
extern crate weaver_error;
extern crate weaver_index;
extern crate weaver_web;
use std::io::{self, Write};


mod cli;
mod app;

fn main() {
    // Setup the logger on the env variable WEAVER.
    // This allows one to do `export WEAVER=debug` to get a lot more errors.
    use std::env;
    use env_logger::{Builder, Target};

    // Use the builder api for more flexibility.
    let mut builder = Builder::new();
    // send output to stderr
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
    let _ = io::stdout().flush();
    info!("weaver-server exited normally");
}
