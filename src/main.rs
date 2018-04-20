// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate chrono;
extern crate clap;
extern crate clipboard;
extern crate cursive;
extern crate cursive_table_view;
#[macro_use]
extern crate diesel;
extern crate daemonize;
extern crate env_logger;
#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hyper;
#[macro_use]
extern crate log;
extern crate mime;
extern crate serde;
extern crate sys_info;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate termion;
extern crate walkdir;

// Re-export the error types for the overall app.
pub use errors::*;

// `error_chain!` creates.
mod errors {

    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {
        foreign_links {
            Diesel(::diesel::result::Error);
            SysInfo(::sys_info::Error);
        }
    }

}


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
mod config;
mod display;
mod entities;
mod controllers;
mod server;
mod store;

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
