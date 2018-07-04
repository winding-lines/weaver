//! File paths, url components and other global configuration information.

pub mod file_utils;
pub mod net;
mod environment;
mod server_config;
pub mod content;
pub mod db;

pub use self::environment::Environment;
pub use self::content::{Channel, Content, OutputKind};
pub use self::server_config::ServerConfig;


/// Destination for the `weaver` CLI.
#[derive(Clone)]
pub enum Destination {
    Remote(String),
}




