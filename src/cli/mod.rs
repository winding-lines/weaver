/// Provides an interface to the command line options.

pub use self::parse::{Command, DataSubCommand, ServerSubCommand, parse};

pub const APP_NAME: &'static str = env!["CARGO_PKG_NAME"];

mod parse;
mod build;
