/// Page content as it is received from the Chrome Extension and serialized in the encrypted repo.

#[derive(Debug, Serialize, Deserialize)]
pub struct PageContent {
    pub url: String,
    pub title: String,
    pub body: String,
}