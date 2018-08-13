/// Provides an interface to the command line options.
pub use self::parse::{parse, Command, CommandAndConfig};

pub const APP_NAME: &str = env!["CARGO_PKG_NAME"];

mod build;
mod parse;
