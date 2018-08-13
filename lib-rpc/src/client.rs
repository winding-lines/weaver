use lib_error::Result;
use lib_goo::config::net::{self, ANNOTATIONS, EPICS};
use lib_goo::config::Destination;
use lib_goo::entities::{Epic, NewAction};
use reqwest;

/// Extract the rpc address from the Destination.
fn rpc_addr(destination: &Destination) -> &str {
    match destination {
        Destination::Remote(ref a) => a,
    }
}

/// Build the query string for recommendations.
/// TODO: use crate to get proper escaping.
fn build_query_string(params: &net::RecommendationQuery) -> String {
    let mut query_string = String::new();

    if let Some(start) = params.start {
        query_string.push_str(&format!("start={}", start));
    }
    if let Some(length) = params.length {
        if !query_string.is_empty() {
            query_string.push('&');
        }
        query_string.push_str(&format!("length={}", length));
    }
    if let Some(ref term) = params.term {
        if !query_string.is_empty() {
            query_string.push('&');
        }
        query_string.push_str(&format!("term={}", term));
    }

    query_string
}

pub fn recommendations(
    destination: &Destination,
    params: &net::RecommendationQuery,
) -> Result<net::PaginatedActions> {
    let url = format!(
        "http://{}{}{}?{}",
        rpc_addr(destination),
        net::ACTIONS2_BASE,
        net::RECOMMENDATIONS,
        build_query_string(params)
    );
    debug!("Downloading recommendations from {}", url);
    let client = reqwest::Client::new();
    let mut response = client
        .get(&url)
        .send()
        .map_err(|e| format!("error in getting recommendations {:?}", e))?;
    response
        .json::<net::PaginatedActions>()
        .map_err(|e| e.into())
}

pub fn add(destination: &Destination, req: &NewAction) -> Result<u64> {
    let client = reqwest::Client::new();
    client
        .post(&format!(
            "http://{}{}",
            rpc_addr(destination),
            net::ACTIONS2_BASE
        ))
        .json(req)
        .send()
        .map(|_| 0)
        .map_err(|e| e.into())
}

pub fn set_annotation(destination: &Destination, id: u64, content: &str) -> Result<u64> {
    let data = net::Annotation {
        annotation: content.into(),
    };
    let client = reqwest::Client::new();
    client
        .post(&format!(
            "http://{}/{}/{}{}",
            rpc_addr(destination),
            net::ACTIONS2_BASE,
            id,
            ANNOTATIONS
        ))
        .json(&data)
        .send()
        .map(|_| 0)
        .map_err(|e| e.into())
}

pub fn fetch_epics(destination: &Destination) -> Result<Vec<Epic>> {
    reqwest::get(&format!("http://{}{}", rpc_addr(destination), EPICS))?
        .json::<Vec<Epic>>()
        .map_err(|e| e.into())
}

pub fn save_epic(destination: &Destination, req: &Epic) -> Result<()> {
    let client = reqwest::Client::new();
    client
        .post(&format!("http://{}{}", rpc_addr(destination), EPICS))
        .json(req)
        .send()
        .map(|_| ())
        .map_err(|e| e.into())
}

pub fn epic(destination: &Destination) -> Result<Epic> {
    reqwest::get(&format!(
        "http://{}{}?query=latest",
        rpc_addr(destination),
        EPICS
    ))?.json::<Epic>()
        .map_err(|e| e.into())
}
