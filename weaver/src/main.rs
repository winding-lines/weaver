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
            print!(" ERR `export WEAVER=error` for more ");
        };
        ::log::error!("error {}", e);

        e.display();

        ::std::process::exit(1);
    }
    ::log::info!("weaver exited normally");
}
