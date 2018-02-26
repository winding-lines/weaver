// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

extern crate chrono;
extern crate clap;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate env_logger;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate walkdir;

// Re-export the error types for the overall app.
pub use errors::*;

// `error_chain!` creates.
mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {}
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
mod entities;
mod controllers;
mod store;

fn main() {
    if let Err(ref e) = controllers::app::run() {
        println!("error: {}", e);

        for e in e.iter().skip(1) {
            println!("caused by: {}", e);
        }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            println!("backtrace: {:?}", backtrace);
        }

        ::std::process::exit(1);
    }
}
