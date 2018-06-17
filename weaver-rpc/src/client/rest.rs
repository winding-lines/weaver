use reqwest;
use weaver_db::entities::{FormattedAction, NewAction};
use weaver_error::Result;
use weaver_db::config::net::{self, ACTIONS_BASE, ANNOTATIONS};

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
    let data = net::Annotation {annotation : content.into()};
    let client = reqwest::Client::new();
    client.post(&format!("http://{}/{}/{}{}", rpc_addr, ACTIONS_BASE, id, ANNOTATIONS))
        .json(&data)
        .send()
        .map(|_| 0)
        .map_err(|e| e.into())
}

pub fn fetch_epics(rpc_addr: &str) -> Result<Vec<String>> {
    unimplemented!()
}
