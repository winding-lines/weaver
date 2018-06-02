use std::sync::Arc;
use weaver_db::RealStore;
use weaver_index::Indexer;

pub(crate) struct AppState {
    pub store: Arc<RealStore>,
    pub indexer: Arc<Indexer>,
}

