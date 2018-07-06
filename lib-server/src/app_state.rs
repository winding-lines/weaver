use std::sync::Arc;
use tera;
use lib_db::RealStore;
use lib_index::{Indexer, Repo};
use lib_error::Result;

/// Store per request state.
pub(crate) struct AppState {
    pub store: Arc<RealStore>,
    pub indexer: Arc<Indexer>,
    pub repo: Arc<Repo>,
    pub template: Result<tera::Tera>,
}


