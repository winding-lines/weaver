//! Shell cli interface to gather information about the shell commands you type
//! and allow you to access the system. You need to hook this into your shell
//! in order to get the benefits, see the README.md file.
//!
//!
//! To get information about the command line options run:
//!
//!    `weaver --help`
//!
//!
//! To execute from the build environment run with
//!
//! `cargo run -p weaver -- --help`
//!
//! The shell process manages the current epic in a local json store and reads the rest
//! of the information from the [weaver-server](../../weaver_server/index.html).
//!
extern crate chrono;
extern crate clap;
extern crate clipboard;
extern crate crossbeam_channel;
extern crate cursive;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate termion;
extern crate walkdir;
// Workspace crates
extern crate lib_error;
extern crate lib_goo;
extern crate lib_rpc;
extern crate lib_tui;

mod api;
mod cli;
mod controllers;
mod display;
mod local_store;

fn main() {
    // Setup the logger on the env variable WEAVER.
    // This allows one to do `export WEAVER=debug` to get a lot more errors.
    use env_logger::{Builder, Target};
    use std::env;

    // Use the builder api for more flexibility.
    let mut builder = Builder::new();
    // send output to stderr in order to be able to debug Cursive layer
    builder.target(Target::Stderr);
    let has_weaver_env = env::var("WEAVER").is_ok();
    if has_weaver_env {
        builder.parse(&env::var("WEAVER").unwrap());
    }
    builder.init();

    // Run the main loop, be concise with error reporting since we may run in PS1.
    if let Err(ref e) = controllers::app::run() {
        if !has_weaver_env {
            print!(" ERR `export WEAVER=error` for more");
        };
        error!("error {}", e);

        for e in e.iter().skip(1) {
            error!("caused by: {}", e);
        }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            error!("backtrace: {:?}", backtrace);
        }
        if !has_weaver_env {
            error!(" `export WEAVER=debug` for more details.");
        }

        ::std::process::exit(1);
    }
    info!("weaver exited normally");
}
