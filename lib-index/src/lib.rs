//! Document repo providing full text search for [Weaver Project](../weaver_project/index.html) and
//! storing documents for further data mining.


pub use crate::indexer::Results;
pub use crate::indexer::{Indexer, TantivyIndexer};
use lib_error::*;

mod indexer;
pub mod repo;

/// Initialize libraries required by the stores.
pub fn init() -> Result<()> {
    if rust_sodium::init().is_err() {
        return Err("crypto init".into());
    };
    Ok(())
}
