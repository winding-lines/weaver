use analyses::Analysis;
use lib_db::SqlProvider;
use lib_error::Result;
use lib_index::repo::Repo;
use lib_index::Indexer;
use std::sync::Arc;
use tera;

/// Store per request state.
pub(crate) struct AppState {
    pub sql: Arc<SqlProvider>,
    pub indexer: Arc<Indexer>,
    pub repo: Arc<Repo>,
    pub template: Result<tera::Tera>,
    pub analyses: Option<Vec<Analysis>>,
}
