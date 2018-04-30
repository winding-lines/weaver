use weaver_db::{actions2, epics, Destination};
use weaver_db::entities::{FormattedAction, NewAction};
use weaver_error::{Result};
use weaver_rpc::client;


pub fn history<T: Into<String>>(_epic: Option<T>, destination: &Destination) -> Result<Vec<FormattedAction>> {
    match *destination {
        Destination::Local(Ok(ref connection)) =>
            actions2::fetch_all(connection),
        Destination::Local(Err(_)) => Err("bad connection".into()),
        Destination::Remote(ref rpc_addr) =>
            client::history(_epic, rpc_addr)
    }
}

// Return the last url action from the store.
pub fn last_url(destination: &Destination) -> Result<Option<(String, String)>> {
    match *destination {
        Destination::Local(Ok(ref connection)) =>
            actions2::last_url(connection),
        Destination::Local(Err(_)) => Err("bad connection".into()),
        Destination::Remote(ref rpc_addr) =>
            client::last_url(rpc_addr)
    }
}

pub fn insert_action(new_action: NewAction, destination: &Destination) -> Result<u64> {
    match *destination {
        Destination::Local(Ok(ref connection)) =>
            actions2::insert(connection, new_action),
        Destination::Local(Err(_)) => Err("bad connection".into()),
        Destination::Remote(ref rpc_addr) =>
            client::add(new_action, rpc_addr)
    }
}

pub fn epic_names(destination: &Destination) -> Result<Vec<String>> {
    match *destination {
        Destination::Local(Ok(ref connection)) =>
            epics::fetch_all(&connection),
        Destination::Local(Err(_)) => Err("bad connection".into()),
        Destination::Remote(ref rpc_addr) =>
            client::fetch_epics(rpc_addr)
    }
}

