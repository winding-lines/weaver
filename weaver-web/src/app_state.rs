use std::sync::Arc;
use tera;
use weaver_db::RealStore;
use weaver_index::{Indexer, Repo};
use weaver_error::Result;

pub(crate) struct AppState {
    pub store: Arc<RealStore>,
    pub indexer: Arc<Indexer>,
    pub repo: Arc<Repo>,
    pub template: Result<tera::Tera>,
}


