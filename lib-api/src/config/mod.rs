pub mod file_utils;
pub mod net;
mod environment;
mod server_config;
pub mod content;
pub mod db;

pub use self::environment::Environment;
pub use self::content::{Channel, Content, OutputKind};
pub use self::server_config::ServerConfig;


#[derive(Clone)]
pub enum Destination {
    Remote(String),
}




