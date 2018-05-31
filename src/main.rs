extern crate chrono;
extern crate clap;
extern crate clipboard;
extern crate cursive;
extern crate cursive_table_view;
extern crate daemonize;
extern crate env_logger;
extern crate futures;
extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate mime;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate termion;
extern crate walkdir;

// Workspace crates
extern crate weaver_db;
extern crate weaver_error;
extern crate weaver_rpc;
extern crate weaver_index;
extern crate weaver_web;

/// A command line tool to create, discover and recommend flows.
///
/// A flow is a sequence of actions towards a mileston. At a given point
/// an user may be:
///
/// - outside of any flow and just logging actions to help identify future flows
/// - in a given flow
/// - look for the next flow.
///

mod cli;
mod display;
mod controllers;
mod http_server;
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
}
