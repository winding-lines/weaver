//! Document repo providing full text search for [Weaver Project](../weaver_project/index.html) and
//! storing documents for further data mining.

extern crate rust_sodium;
extern crate metrohash;
extern crate keyring;
extern crate rpassword;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate tantivy;
extern crate lib_api;
extern crate lib_error;

pub use indexer::Indexer;
pub use indexer::Results;
use lib_error::*;

mod indexer;
mod repo;
mod config;
pub use repo::Repo;

/// Initialize libraries required by the stores.
pub fn init() -> Result<()> {
    if rust_sodium::init().is_err() {
        return Err("crypto init".into())
    };
    Ok(())
}

