use lib_error::*;
use lib_goo::config::net::{self, ANNOTATIONS, EPICS};
use lib_goo::config::Destination;
use lib_goo::entities::{Epic, NewAction, ActionId};
use reqwest;
use serde_urlencoded;

/// Extract the rpc address from the Destination.
fn rpc_addr(destination: &Destination) -> &str {
    match destination {
        Destination::Remote(ref a) => a,
    }
}

pub fn recommendations(
    destination: &Destination,
    params: &net::RecommendationQuery,
) -> Result<net::PaginatedActions> {
    let url = format!(
        "http://{}{}{}{}?{}",
        rpc_addr(destination),
        net::API_BASE,
        net::ACTIONS2_BASE,
        net::RECOMMENDATIONS,
        serde_urlencoded::to_string(params).chain_err(|| "generate recommendation url")?
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
            "http://{}{}{}",
            rpc_addr(destination),
            net::API_BASE,
            net::ACTIONS2_BASE
        ))
        .json(req)
        .send()
        .map(|_| 0)
        .map_err(|e| e.into())
}

pub fn set_annotation(destination: &Destination, id: &ActionId, content: &str) -> Result<u64> {
    let data = net::Annotation {
        annotation: content.into(),
    };
    let client = reqwest::Client::new();
    client
        .post(&format!(
            "http://{}{}/{}/{}{}",
            rpc_addr(destination),
            net::API_BASE,
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
