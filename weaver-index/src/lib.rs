//! Document repo providing full text search for [Weaver](../weaver/index.html) and
//! storing documents for further data mining.

extern crate rust_sodium;
extern crate metrohash;
extern crate keyring;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate bincode;
extern crate tantivy;
extern crate weaver_db;
extern crate weaver_error;

pub use indexer::Indexer;
pub use indexer::Results;
use weaver_error::*;

mod indexer;
mod repo;
mod config;
pub use repo::Repo;

pub fn init() -> Result<Repo> {
    if rust_sodium::init().is_err() {
        return Err("crypto init".into())
    };
    Repo::build()
}

