//! File paths, url components and other global configuration information.

pub mod db;
mod environment;
pub mod file_utils;
pub mod net;
mod output_kind;
mod server_config;

pub use self::environment::Environment;
pub use self::output_kind::{Channel, OutputKind};
pub use self::server_config::ServerConfig;

/// Destination for the `weaver` CLI.
#[derive(Clone)]
pub enum Destination {
    Remote(String),
}
