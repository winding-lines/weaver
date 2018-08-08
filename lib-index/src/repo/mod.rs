use lib_error::*;
use std::convert::From;

mod encrypted_repo;
mod config;
pub use self::encrypted_repo::EncryptedRepo;

/// Represents a collection in the repo.
#[derive(Debug)]
pub struct Collection(pub String);

impl Collection {
    fn name(&self) -> &str {
        &self.0
    }
}

impl From<String> for Collection {
    fn from(name: String) -> Self {
        Collection(name)
    }
}

impl<'a> From<&'a str> for Collection {
    fn from(name: &'a str) -> Self {
        Collection(name.into())
    }
}

/// Trait with document management related api
pub trait Repo {
    fn add(&self, collection: &Collection, content: &[u8]) -> Result<String>;
}
