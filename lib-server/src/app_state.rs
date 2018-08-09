use analyses::Analysis;
use lib_db::SqlProvider;
use lib_error::Result;
use lib_index::repo::Repo;
use lib_index::Indexer;
use std::sync::Arc;
use tera;

/// Store per request state.
pub(crate) struct AppState {
    pub analyses: Option<Vec<Analysis>>,
    pub indexer: Arc<Indexer>,
    pub repo: Arc<Repo>,
    pub sql: Arc<SqlProvider>,
    pub template: Result<tera::Tera>,
}

// Define a helper environment for tests.
#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use lib_db::{Connection, SqlProvider};
    use lib_error::Result as WResult;
    use lib_goo::entities::PageContent;
    use lib_index::repo::Collection;
    use lib_index::repo::Repo;
    use lib_index::{Indexer, Results};
    use pages;
    use std::cell::RefCell;
    use std::sync::Arc;

    struct FailingSqlProvider;

    impl SqlProvider for FailingSqlProvider {
        fn connection(&self) -> WResult<Connection> {
            return Err("sqlprovider not implemented for testing".into());
        }
    }

    struct TestIndexer {
        pages: RefCell<Vec<PageContent>>,
    }

    impl TestIndexer {
        fn new() -> Self {
            Self {
                pages: RefCell::new(Vec::new()),
            }
        }
    }

    impl Indexer for TestIndexer {
        fn add(&self, page_content: &PageContent) -> WResult<(u64)> {
            self.pages.borrow_mut().push(page_content.clone());
            Ok(1)
        }

        fn delete(&self, _id: &str) -> WResult<()> {
            self.pages.borrow_mut().clear();
            Ok(())
        }
        fn search(&self, _what: &str) -> WResult<Results> {
            Ok(Results {
                total: 45,
                matches: self.pages.borrow().clone(),
            })
        }
        fn summary(&self) -> Option<String> {
            Some("soomary".into())
        }
    }

    struct TestRepo;

    impl Repo for TestRepo {
        fn add(&self, _collection: &Collection, _content: &[u8]) -> WResult<String> {
            Ok("repo add".into())
        }
    }

    pub(crate) fn default_test() -> AppState {
        AppState {
            analyses: None,
            indexer: Arc::new(TestIndexer::new()),
            repo: Arc::new(TestRepo),
            sql: Arc::new(FailingSqlProvider),
            template: pages::build_tera(),
        }
    }
}
