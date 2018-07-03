//! Web and api server for [Weaver](../weaver/index.html).

extern crate actix_web;
extern crate bytes;
extern crate futures;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate bincode;
extern crate tera;

extern crate weaver_db;
extern crate lib_api;
extern crate weaver_error;
extern crate weaver_index;


pub use server::Server;

mod app_state;
mod server;
mod handlers;
mod pages;


