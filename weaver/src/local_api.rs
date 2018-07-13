//! Implement the API to be used on the client. By default it will either use the server
//! but can also use the local database, if available.

use lib_goo::config::{Destination, Environment};
use lib_goo::entities::{Epic, FormattedAction, NewAction};
use lib_error::Result;
use lib_rpc::client;


// Macro to dispatch to local or remote api
//
// It takes 5 arguments:
//   - destination
//   - name (with optional module) of the local function
//   - additional arguments for the local function, the first will be the database connection
//   - name (with optional module) of the remote function
//   - additional arguments for the remote function, the first one will be the remote address

macro_rules! dispatch {
    ($dest:expr, $remote:path, ($($rarg:tt)*) ) => ({
        match *$dest {
            Destination::Remote(ref rpc_addr) =>
                $remote(rpc_addr, $($rarg)* )
        }
    })
}

pub fn history(_env: &Environment, destination: &Destination) -> Result<Vec<FormattedAction>> {
    dispatch!(destination,
        client::rest::history, ())
}

pub fn insert_action(new_action: &NewAction, destination: &Destination) -> Result<u64> {
    dispatch!(destination,
            client::rest::add, (new_action)
    )
}

pub fn epic_names(destination: &Destination) -> Result<Vec<Epic>> {
    dispatch!(destination,
            client::rest::fetch_epics, ()
    )
}

#[allow(dead_code)]
pub fn set_annotation(destination: &Destination, id: u64, content: &str) -> Result<u64> {
    dispatch!(destination,
        client::rest::set_annotation, (id, content)
    )
}


