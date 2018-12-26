//! Web and api server for [Weaver](../weaver/index.html).


pub use crate::server::Server;

mod analyses;
mod asset_map;
mod app_state;
mod handlers;
mod pages;
mod server;
mod template_engine;
