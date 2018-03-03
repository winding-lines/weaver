/// Provides an interface to the command line options.

pub use self::parse::{Command, parse};

pub const APP_NAME: &'static str = env!["CARGO_PKG_NAME"];
pub const APP_FOLDER: &'static str = ".weaver";

mod parse;
mod build;
