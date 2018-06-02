use std::sync::Arc;
use tera;
use weaver_db::RealStore;
use weaver_index::Indexer;
use weaver_error::Result;

pub(crate) struct AppState {
    pub store: Arc<RealStore>,
    pub indexer: Arc<Indexer>,
    pub template: Result<tera::Tera>,
}


