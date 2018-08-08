/// Integrate with the LDA output from weaver-recommend LDA.

// Top level definition of the doc-topics.json file
#[derive(Debug,Deserialize)]
pub struct DocTopics {
    pub topics: Vec<Topic>,
    pub documents: Vec<Doc>
}

#[derive(Debug,Deserialize)]
pub struct Topic {
    pub words: Vec<TopicWord>,
}

// For each topic give the words and the expectation of the given words.
#[derive(Debug,Deserialize)]
pub struct TopicWord {
    pub w: String,
    pub e: f32
}

// For each document map the url to the most relevant topics
#[derive(Debug,Deserialize)]
pub struct Doc {
    pub url: String,
    pub relevant: Vec<RelTopic>,
}

#[derive(Debug,Deserialize)]
pub struct RelTopic {
    pub t: usize,
    pub p: f32,
}

