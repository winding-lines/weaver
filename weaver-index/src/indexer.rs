use std::fs;
use tantivy::collector::TopCollector;
use tantivy::Index;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use weaver_db::config::file_utils::app_folder;
use weaver_error::*;

pub struct Indexer {
    index: Index,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Results {
    pub total: u64,
    pub matches: Vec<(String, String)>,
}

impl Indexer {
    pub fn build() -> Result<Indexer> {
        let mut index_path = app_folder()?;
        index_path.push("text-index");

        if !index_path.exists() {
            fs::create_dir(&index_path).chain_err(|| "create index folder")?;
            let mut schema_builder = SchemaBuilder::default();

            schema_builder.add_text_field("id", STRING | STORED);

            schema_builder.add_text_field("title", TEXT | STORED);

            schema_builder.add_text_field("body", TEXT);

            let schema = schema_builder.build();
            let _ = Index::create(index_path.clone(), schema)
                .chain_err(|| "create index")?;
        }


        let index = Index::open(index_path)
            .chain_err(|| "open index")?;

        Ok(Indexer {
            index
        })
    }

    pub fn add(&self, id: &str, title: &str, body: &str) -> Result<(u64)> {
        let mut index_writer = self.index.writer(50_000_000)
            .chain_err(|| "create index writer")?;

        let schema = self.index.schema();
        let f_id = schema.get_field("id")
            .chain_err(|| "get id field")?;
        let f_title = schema.get_field("title")
            .chain_err(|| "get title field")?;
        let f_body = schema.get_field("body")
            .chain_err(|| "get body field")?;
        let term = Term::from_field_text(f_id, id);
        index_writer.delete_term(term);
        let mut doc = Document::default();
        doc.add_text(f_id, id);
        doc.add_text(f_title, title);
        doc.add_text(f_body, body);
        index_writer.add_document(doc);
        index_writer.commit()
            .chain_err(|| "commit index")
    }

    pub fn search(&self, what: &str) -> Result<Results> {
        self.index.load_searchers()
            .chain_err(|| "load searchers")?;

        // Afterwards create one (or more) searchers.
        //
        // You should create a searcher
        // every time you start a "search query".
        let searcher = self.index.searcher();

        let schema = self.index.schema();
        let f_id = schema.get_field("id")
            .chain_err(|| "get id field")?;
        let f_title = schema.get_field("title")
            .chain_err(|| "get title field")?;
        let f_body = schema.get_field("body")
            .chain_err(|| "get body field")?;
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
            Err(e) => return Err(format!("error parsing query {:?}", e).into())
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
        // only in the top 10. Keeping track of our top 40 best documents
        // is the role of the TopCollector.
        let mut top_collector = TopCollector::with_limit(40);

        // We can now perform our query.
        searcher.search(&*query, &mut top_collector)
            .chain_err(|| "actual search")?;

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
            let retrieved_doc = searcher.doc(&doc_address)
                .chain_err(|| "retrieve document")?;
            let found_id = retrieved_doc.get_first(f_id).map(|a| a.text())
                .unwrap_or("no id");
            let found_title = retrieved_doc.get_first(f_title).map(|a| a.text())
                .unwrap_or("no title");
            out.push((String::from(found_id), String::from(found_title)));
        }
        Ok(Results {
            total: searcher.num_docs(),
            matches: out,
        })
    }
}

