/// Page content as it is received from the Chrome Extension and serialized in the encrypted repo.

#[derive(Debug, Serialize, Deserialize)]
pub struct PageContent {
    pub url: String,
    pub title: String,
    pub body: String,
}

impl PageContent {
    /// Name of the collection to use in the encrypted repo.
    pub fn collection_name() -> &'static str {
        "page-content"
    }
}
