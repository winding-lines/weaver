use lib_error::*;
/// Ports and other information about the server.
use std::net::{TcpListener, ToSocketAddrs};

pub const HTTP_PORT: u16 = 8466;
pub const HTTPS_PORT: u16 = 8467;

pub struct ServerConfig {
    pub http_port: u16,
    pub https_port: u16,
    pub address: String,
    pub base_url: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            http_port: HTTP_PORT,
            https_port: HTTPS_PORT,
            address: String::from("127.0.0.1"),
            base_url: String::from("/"),
        }
    }
}

impl ServerConfig {
    pub fn current() -> ServerConfig {
        ServerConfig::default()
    }

    pub fn actix_address(&self) -> String {
        format!("localhost:{}", self.http_port)
    }

    pub fn is_running(&self) -> bool {
        is_listening(&self.actix_address())
    }

    pub fn check(&self) -> Result<()> {
        println!("actix listening {}", self.is_running());
        Ok(())
    }
}

fn is_listening(http_addr: &str) -> bool {
    let addr = match http_addr.to_socket_addrs().map(|ref mut i| i.next()) {
        Ok(Some(a)) => a,
        Ok(_) | Err(_) => panic!("unable to resolve listener address {}", http_addr),
    };

    match TcpListener::bind(addr) {
        Ok(listener) => {
            // We were able to bind to the address => no server is listening.
            drop(listener);
            false
        }
        Err(_) => {
            ::log::debug!(
                "Error binding to {}, assume the server is running.",
                http_addr
            );
            true
        }
    }
}
