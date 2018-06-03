/// Provides an interface to the command line options.

pub use self::parse::{Command, CommandAndConfig, DataSubCommand, parse, ServerSubCommand};

pub const APP_NAME: &str = env!["CARGO_PKG_NAME"];

pub const RPC_ADDRESS: &str = "127.0.0.1:8465";
pub const ACTIX_ADDRESS: &str = "127.0.0.1:8466";

pub struct ServerConfig {
    pub rpc_address: String,
    pub actix_address: String,
}


impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            rpc_address: RPC_ADDRESS.into(),
            actix_address: ACTIX_ADDRESS.into(),
        }
    }
}

impl ServerConfig {
    pub fn current() -> ServerConfig {
        ServerConfig::default()
    }
}

mod parse;
mod build;
