extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tantivy;
extern crate weaver_db;
extern crate weaver_error;


pub use indexer::Indexer;
pub use indexer::Results;

mod indexer;

