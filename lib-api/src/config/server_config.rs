use std::net::{TcpListener, ToSocketAddrs};
use weaver_error::*;

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

    pub fn is_running(&self) -> bool {
        is_listening(&self.actix_address)
    }

    pub fn check(&self) -> Result<()> {
        println!("actix listening {}", self.is_running());
        Ok(())
    }
}

fn is_listening(http_addr: &str) -> bool {
    let addr = match http_addr.to_socket_addrs().map(|ref mut i| i.next()) {
        Ok(Some(a)) => a,
        Ok(_) | Err(_) => panic!("unable to resolve listener address"),
    };

    match TcpListener::bind(addr) {
        Ok(listener) => {
            // We were able to bind to the address => no server is listening.
            drop(listener);
            false
        }
        Err(_) => {
            debug!("Error binding to {}, assume the server is running.", http_addr);
            true
        }
    }
}


