pub use self::server::Server;

pub mod greeter;
pub mod historian;
#[cfg_attr(feature = "cargo-clippy", allow(module_inception))]
mod server;


