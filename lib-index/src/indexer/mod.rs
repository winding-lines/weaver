use lib_error::*;
use lib_goo::entities::PageContent;

mod tantivy_indexer;

pub use self::tantivy_indexer::TantivyIndexer;

#[derive(Serialize, Deserialize, Default)]
pub struct Results {
    pub total: u64,
    pub matches: Vec<PageContent>,
}

/// Public/light interface to the indexer.
pub trait Indexer {
    fn add(&self, page_content: &PageContent) -> Result<(u64)>;
    fn delete(&self, id: &str) -> Result<()>;
    fn search(&self, what: &str) -> Result<Results>;
    fn summary(&self) -> Option<String>;
}
