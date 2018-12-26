use lib_error::*;
use lib_goo::config::net::{self, ANNOTATIONS};
use lib_goo::config::Destination;
use lib_goo::entities::{ActionId, NewAction};
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
        serde_urlencoded::to_string(params).context("encoding url params".into())?
    );
    ::log::debug!("Downloading recommendations from {}", url);
    let client = reqwest::Client::new();
    let mut response = client
        .get(&url)
        .send()
        .context("error in getting recommendations".into())?;
    response
        .json::<net::PaginatedActions>().map_err(|a| a.into())
}

pub fn add(destination: &Destination, req: &NewAction) -> Result<u64> {
    let client = reqwest::Client::new();
    let id = client
        .post(&format!(
            "http://{}{}{}",
            rpc_addr(destination),
            net::API_BASE,
            net::ACTIONS2_BASE
        )).json(req)
        .send()
        .map(|_| 0)?;
    Ok(id)
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
        )).json(&data)
        .send()
        .map(|_| 0)
        .map_err(|a| a.into())
}
