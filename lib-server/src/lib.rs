//! Web and api server for [Weaver](../weaver/index.html).

extern crate actix_web;
extern crate bincode;
extern crate bson;
extern crate bytes;
extern crate futures;
extern crate inflections;
extern crate lib_ai;
extern crate lib_db;
extern crate lib_error;
extern crate lib_goo;
extern crate lib_index;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tera;
extern crate walkdir;

pub use server::Server;

mod analyses;
mod app_state;
mod handlers;
mod pages;
mod server;
mod template_engine;
