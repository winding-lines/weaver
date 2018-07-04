use lib_api::config::net::{self, ACTIONS_BASE, ANNOTATIONS, EPICS};
use lib_api::entities::{Epic, FormattedAction, NewAction};
use reqwest;
use lib_error::Result;

pub fn history(rpc_addr: &str) -> Result<Vec<FormattedAction>> {
    reqwest::get(&format!("http://{}{}", rpc_addr, ACTIONS_BASE))?
        .json::<Vec<FormattedAction>>()
        .map_err(|e| e.into())
}


pub fn add(rpc_addr: &str, req: &NewAction) -> Result<u64> {
    let client = reqwest::Client::new();
    client.post(&format!("http://{}{}", rpc_addr, ACTIONS_BASE))
        .json(req)
        .send()
        .map(|_| 0)
        .map_err(|e| e.into())
}

pub fn set_annotation(rpc_addr: &str, id: u64, content: &str) -> Result<u64> {
    let data = net::Annotation { annotation: content.into() };
    let client = reqwest::Client::new();
    client.post(&format!("http://{}/{}/{}{}", rpc_addr, ACTIONS_BASE, id, ANNOTATIONS))
        .json(&data)
        .send()
        .map(|_| 0)
        .map_err(|e| e.into())
}

pub fn fetch_epics(rpc_addr: &str) -> Result<Vec<Epic>> {
    reqwest::get(&format!("http://{}{}", rpc_addr, EPICS))?
        .json::<Vec<Epic>>()
        .map_err(|e| e.into())
}

pub fn save_epic(rpc_addr: &str, req: &Epic) -> Result<()> {
    let client = reqwest::Client::new();
    client.post(&format!("http://{}{}", rpc_addr, EPICS))
        .json(req)
        .send()
        .map(|_| ())
        .map_err(|e| e.into())
}

pub fn epic(rpc_addr: &str) -> Result<Epic> {
    reqwest::get(&format!("http://{}{}?query=latest", rpc_addr, EPICS))?
        .json::<Epic>()
        .map_err(|e| e.into())
}
