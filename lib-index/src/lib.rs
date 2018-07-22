//! Document repo providing full text search for [Weaver Project](../weaver_project/index.html) and
//! storing documents for further data mining.

extern crate keyring;
#[macro_use]
extern crate log;
extern crate metrohash;
extern crate rpassword;
extern crate rust_sodium;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate lib_error;
extern crate lib_goo;
extern crate tantivy;

pub use indexer::Indexer;
pub use indexer::Results;
use lib_error::*;

mod config;
mod indexer;
pub mod repo;
pub use repo::Repo;

/// Initialize libraries required by the stores.
pub fn init() -> Result<()> {
    if rust_sodium::init().is_err() {
        return Err("crypto init".into());
    };
    Ok(())
}
