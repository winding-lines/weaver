/// Provides an interface to the command line options.

pub use self::parse::{Command, CommandAndConfig, DataSubCommand, ServerSubCommand, parse};

pub const APP_NAME: &'static str = env!["CARGO_PKG_NAME"];

pub const HTTP_ADDRESS: &'static str = "127.0.0.1:8464";
pub const RPC_ADDRESS: &'static str = "127.0.0.1:8465";

pub struct ServerConfig {
    pub http_address: String,
    pub rpc_address: String,
}


impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            http_address: HTTP_ADDRESS.into(),
            rpc_address: RPC_ADDRESS.into(),
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
