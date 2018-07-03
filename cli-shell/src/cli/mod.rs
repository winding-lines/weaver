/// Provides an interface to the command line options.
pub use self::parse::{Command, CommandAndConfig, parse};

pub const APP_NAME: &str = env!["CARGO_PKG_NAME"];

mod parse;
mod build;



