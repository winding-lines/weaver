/// Page content as it is received from the Chrome Extension and serialized in the encrypted repo.

#[derive(Clone, Debug, ::serde::Serialize, ::serde::Deserialize, Default)]
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
