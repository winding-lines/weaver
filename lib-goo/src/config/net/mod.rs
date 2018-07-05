pub const ACTIONS_BASE: &str = "/actions";
pub const ANNOTATIONS: &str = "/annotations";
pub const EPICS: &str = "/epics";

#[derive(Deserialize)]
pub struct Pagination {
    pub start: Option<usize>,
    pub length: Option<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct Annotation {
    pub annotation: String,
}

