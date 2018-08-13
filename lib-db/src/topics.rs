use lib_error::*;
use lib_goo::config::file_utils;
use lib_goo::entities::lda::*;
use serde_json as json;
use std::collections::HashMap;

/// Hold Topic definition and mapping from documents to topics.
pub struct TopicStore {
    topics: Vec<Topic>,
    doc2topic: HashMap<String, Vec<RelTopic>>,
}

impl TopicStore {
    // Return the topic list for a given url.
    pub fn topics_for_url(&self, url: &str) -> Option<&[RelTopic]> {
        self.doc2topic.get(url).as_ref().map(|a| a.as_slice())
    }

    // Return the topic at the given index.
    pub fn topic_at_ndx(&self, ndx: usize) -> &Topic {
        &self.topics[ndx]
    }

    pub fn build(doc_topics: DocTopics) -> TopicStore {
        let mut doc2topic = HashMap::new();
        for doc in doc_topics.documents {
            doc2topic.insert(doc.url, doc.relevant);
        }
        TopicStore {
            topics: doc_topics.topics,
            doc2topic,
        }
    }

    // Check the status of the TopicStore and display a summary.
    pub fn check() -> Result<()> {
        Self::load().map(|loaded| match loaded {
            None => println!("Document topics missing"),
            Some(ts) => println!(
                "Document topics ok {} documents {} topics",
                ts.doc2topic.len(),
                ts.topics.len()
            ),
        })
    }

    pub fn load() -> Result<Option<TopicStore>> {
        // build the path
        let mut path = file_utils::app_folder()?;
        path.push("analyses");
        path.push("data");
        path.push("doc-topics.json");
        if !path.exists() {
            return Ok(None);
        }

        // load the json
        let contents = file_utils::read_content(&path)?;
        let doc_topics: DocTopics = json::from_str(&contents).chain_err(|| "load doc-topics.json")?;

        // build the store
        Ok(Some(TopicStore::build(doc_topics)))
    }
}
