use lib_db::{topics, SqlProvider};
use lib_index::repo::Repo;
use lib_index::Indexer;
use std::sync::Arc;

/// Store per request state.
pub(crate) struct ApiState {
    pub indexer: Arc<Indexer>,
    pub repo: Arc<Repo>,
    pub sql: Arc<SqlProvider>,
    pub topic_store: Arc<Option<topics::TopicStore>>,
}

// Define a helper environment for tests.
#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use lib_db::actions2;
    use lib_db::test_helpers::SqlStoreInMemory;
    use lib_db::{Connection, SqlProvider};
    use lib_error::{Result as WResult};
    use lib_goo::entities::{NewAction, PageContent};
    use lib_index::repo::Collection;
    use lib_index::repo::Repo;
    use lib_index::{Indexer, Results};
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

    pub(crate) fn default_test() -> ApiState {
        ApiState {
            indexer: Arc::new(TestIndexer::new()),
            repo: Arc::new(TestRepo),
            sql: Arc::new(FailingSqlProvider),
            topic_store: Arc::new(None),
        }
    }

    /// Structure to use when populating the State with sql entities. This is
    /// required because the in memory database disappears when the connection is
    /// closed.
    pub(crate) struct StateWithActions(pub Arc<Vec<String>>);

    impl StateWithActions {
        // The ApiState to use during the tests.
        pub fn state(&self) -> ApiState {
            let mut s = default_test();
            let actions = self.0.clone();
            s.sql = Arc::new(SqlStoreInMemory::build(move |connection| {
                for a in actions.iter() {
                    let one = NewAction {
                        command: a.to_string(),
                        ..NewAction::default()
                    };
                    actions2::insert(&connection, &one)?;
                }
                Ok(())
            }));
            s
        }
    }
}
