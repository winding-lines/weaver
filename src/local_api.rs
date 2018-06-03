//! Implement the API to be used on the client. By default it will either use the server
//! but can also use the local database, if available.

use weaver_db::{actions2, Destination, epics};
use weaver_db::config::Environment;
use weaver_db::entities::{FormattedAction, NewAction};
use weaver_error::Result;
use weaver_rpc::client;

// Macro to dispatch to local or remote api
//
// It takes 5 arguments:
//   - destination
//   - name (with optional module) of the local function
//   - additional arguments for the local function, the first will be the database connection
//   - name (with optional module) of the remote function
//   - additional arguments for the remote function, the first one will be the remote address

macro_rules! dispatch {
    ($dest:expr, $local:path, ($($larg:tt)*), $remote:path, ($($rarg:tt)*) ) => ({
        match *$dest {
            Destination::Local(Ok(ref connection)) =>
                $local(connection, $($larg)*),
            Destination::Local(Err(_)) => Err("bad connection".into()),
            Destination::Remote(ref rpc_addr) =>
                $remote(rpc_addr, $($rarg)* )
        }
    })
}

pub fn history(env: &Environment, destination: &Destination) -> Result<Vec<FormattedAction>> {
    dispatch!(destination,
        actions2::fetch_all, (),
        client::history, (env))
}

pub fn insert_action(new_action: NewAction, destination: &Destination) -> Result<u64> {
    dispatch!(destination,
            actions2::insert, (new_action),
            client::add, (new_action)
    )
}

pub fn epic_names(destination: &Destination) -> Result<Vec<String>> {
    dispatch!(destination,
            epics::fetch_all, (),
            client::fetch_epics, ()
    )
}

pub fn set_annotation(destination: &Destination, id: u64, content: &str) -> Result<u64> {
    dispatch!(destination,
        actions2::set_annotation, (id, content),
        client::set_annotation, (id, content)
    )
}

