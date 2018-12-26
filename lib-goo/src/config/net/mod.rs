//! Url building constants and data structures used over the wire.
//!
use crate::entities::{Cycle, FormattedAction};

pub const API_BASE: &str = "/api";
pub const ACTIONS2_BASE: &str = "/v2/actions";
pub const ANNOTATIONS: &str = "/annotations";
pub const RECOMMENDATIONS: &str = "/recommendations";

/// A request to change the annotation for a given entry.
#[derive(::serde::Serialize, ::serde::Deserialize)]
pub struct Annotation {
    pub annotation: String,
}

/// A request for paginated data.
#[derive(Debug, Default, ::serde::Deserialize, ::serde::Serialize)]
pub struct Pagination {
    /// Offset
    pub start: Option<i64>,
    /// How many to fetch. For sqlite3 -1 means no limit.
    pub length: Option<i64>,
}

/// A paginated response.
#[derive(::serde::Deserialize, ::serde::Serialize)]
pub struct PaginatedActions {
    pub entries: Vec<FormattedAction>,
    pub total: usize,
    pub cycles: Vec<Cycle>,
}

/// Request parameters to fetch recommendations.
#[derive(::serde::Deserialize, ::serde::Serialize)]
pub struct RecommendationQuery {
    pub term: Option<String>,
    pub start: Option<i64>,
    pub length: Option<i64>,
}
