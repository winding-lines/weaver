//! Weaver server, to execute from the build environment run with
//!
//! `cargo run -p cli-server -- check`
//!
//! The server provides a REST interface to the Weaver stores. The interface
//! is used by the shell and chrome extensions.
//!
//! # Development
//!
//! During development you can setup a staging server with the following command
//!
//! `WEAVER=debug cargo run -p weaver-server -- -C weaver-staging -P -p 8888 start -fg`
//!
//! (you can also add `RUST_BACKTRACE=1` in front of the command).
//!
//! The [weaver-data](../weaver_fata/index.html) needs to be setup accordingly.
//!
use std::io::{self, Write};

mod app;
mod cli;

fn main() {
    // Setup the logger on the env variable WEAVER.
    // This allows one to do `export WEAVER=debug` to get a lot more errors.
    use env_logger::{Builder, Target};
    use std::env;

    // Use the builder api for more flexibility.
    let mut builder = Builder::new();
    // send output to stderr
    builder.target(Target::Stderr);

    // Setup the log levels, WEAVER env variable takes precedene.
    if env::var("WEAVER").is_ok() {
        builder.parse(&env::var("WEAVER").unwrap());
    } else {
        builder.filter_module("actix_web", log::LevelFilter::Info);
        builder.filter_module("lib_db", log::LevelFilter::Info);
        builder.filter_module("lib_server", log::LevelFilter::Info);
        builder.filter_module("tantivy", log::LevelFilter::Info);
        builder.filter_level(log::LevelFilter::Error);
    }
    builder.init();

    // Run the main loop, be concise with error reporting since we may run in PS1.
    if let Err(ref e) = app::run() {
        print!(" ERR `export WEAVER=error` for more ");
        ::log::error!("error {}", e);

        e.display();

        ::std::process::exit(1);
    }
    let _ = io::stdout().flush();
    ::log::info!("weaver-server exited normally");
}
