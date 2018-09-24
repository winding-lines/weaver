//! Provide an interface to the Tantivy index.
//!
use crate::indexer::{Indexer, Results};
use lib_error::*;
use lib_goo::config::file_utils::app_folder;
use lib_goo::entities::PageContent;
use std::fs;
use std::path::PathBuf;
use tantivy::collector::TopCollector;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::Index;

pub struct TantivyIndexer {
    index: Index,
}

fn index_path() -> Result<PathBuf> {
    let mut path = app_folder()?;
    path.push("text-index");
    Ok(path)
}

impl TantivyIndexer {
    /// Build the application wide indexer. If the index is not setup properly this will
    /// fail and the user should call the setup function.
    pub fn build() -> Result<Self> {
        let index_path = index_path()?;

        let index = Index::open_in_dir(index_path)?;

        Ok(Self { index })
    }

    /// Delete all the files int the index.
    pub fn delete_all() -> Result<()> {
        let index_path = index_path()?;
        if index_path.exists() {
            fs::remove_dir_all(&index_path)?;
        }
        Ok(())
    }

    /// Setup the index
    pub fn setup_if_needed() -> Result<()> {
        let index_path = index_path()?;
        if !index_path.exists() {
            fs::create_dir(&index_path)?;
            let mut schema_builder = SchemaBuilder::default();

            schema_builder.add_text_field("id", STRING | STORED);

            schema_builder.add_text_field("title", TEXT | STORED);

            schema_builder.add_text_field("body", TEXT);

            let schema = schema_builder.build();
            let _ = Index::create_in_dir(index_path.clone(), schema)?;
        }

        Ok(())
    }

    /// Display information about the repo, returns any errors.
    pub fn check() -> Result<()> {
        let index_path = index_path()?;

        if !index_path.exists() {
            return Err("Index path does not exist".into());
        }
        let indexer = Self::build()?;
        println!("Indexer ok {:?}.", indexer.summary());
        Ok(())
    }
}

impl Indexer for TantivyIndexer {
    fn add(&self, page_content: &PageContent) -> Result<(u64)> {
        let mut index_writer = self.index.writer_with_num_threads(1, 10_000_000)?;

        let schema = self.index.schema();
        let f_id = schema.get_field("id").expect("get id field");
        let f_title = schema.get_field("title").expect("get title field");
        let f_body = schema.get_field("body").expect("get body field");
        let term = Term::from_field_text(f_id, &page_content.url);
        index_writer.delete_term(term);
        let mut doc = Document::default();
        doc.add_text(f_id, &page_content.url);
        doc.add_text(f_title, &page_content.title);
        doc.add_text(f_body, &page_content.body);
        index_writer.add_document(doc);
        let id = index_writer.commit()?;
        Ok(id)
    }

    fn delete(&self, id: &str) -> Result<()> {
        let mut index_writer = self.index.writer(50_000_000)?;

        let schema = self.index.schema();
        let f_id = schema.get_field("id").expect("get id field");
        let term = Term::from_field_text(f_id, id);
        index_writer.delete_term(term);
        index_writer.commit()?;
        Ok(())
    }

    fn search(&self, what: &str) -> Result<Results> {
        self.index.load_searchers()?;

        // Afterwards create one (or more) searchers.
        //
        // You should create a searcher
        // every time you start a "search query".
        let searcher = self.index.searcher();

        let schema = self.index.schema();
        let f_id = schema.get_field("id").expect("get id field");
        let f_title = schema.get_field("title").expect("get title field");
        let f_body = schema.get_field("body").expect("get body field");
        // The query parser can interpret human queries.
        // Here, if the user does not specify which
        // field they want to search, tantivy will search
        // in both title and body.
        let query_parser = QueryParser::for_index(&self.index, vec![f_title, f_body]);

        // QueryParser may fail if the query is not in the right
        // format. For user facing applications, this can be a problem.
        // A ticket has been opened regarding this problem.
        let query = match query_parser.parse_query(what) {
            Ok(q) => q,
            Err(e) => return Err(WeaverError::from(format!("error parsing query {:?}", e))),
        };

        // A query defines a set of documents, as
        // well as the way they should be scored.
        //
        // A query created by the query parser is scored according
        // to a metric called Tf-Idf, and will consider
        // any document matching at least one of our terms.

        // ### Collectors
        //
        // We are not interested in all of the documents but
        // only in the top N. Keeping track of our top best documents
        // is the role of the TopCollector.
        let mut top_collector = TopCollector::with_limit(40);

        // We can now perform our query.
        searcher.search(&*query, &mut top_collector)?;

        // Our top collector now contains the 10
        // most relevant doc ids...
        let doc_addresses = top_collector.docs();

        // The actual documents still need to be
        // retrieved from Tantivy's store.
        //
        // Since the body field was not configured as stored,
        // the document returned will only contain
        // a title.

        let mut out = Vec::new();
        for doc_address in doc_addresses {
            let retrieved_doc = searcher.doc(doc_address)?;
            let found_id = retrieved_doc
                .get_first(f_id)
                .expect("missing id in retrieved document")
                .text()
                .unwrap_or_default();
            let found_title = retrieved_doc
                .get_first(f_title)
                .expect("missing title in retrieved document")
                .text()
                .unwrap_or_default();
            out.push(PageContent {
                url: String::from(found_id),
                title: String::from(found_title),
                body: String::new(),
            });
        }
        Ok(Results {
            total: searcher.num_docs(),
            matches: out,
        })
    }

    /// Build a textual representation of the summary to be displayed in the web interface.
    fn summary(&self) -> Option<String> {
        self.search("weaver")
            .map(|r| format!("Indexed docs: {}", r.total))
            .ok()
    }
}
