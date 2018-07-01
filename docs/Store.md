# Description

Weaver stores data on the local disk. One of the research goals of the project
is to make this data as secured as possible.

# Structure

Information is stored in 3 backends: the raw repo, sqlite, and tantivy.

## Raw repo

The information is stored as unprocessed records as it comes in.
The serialization is done with [bincode](https://crates.io/crates/bincode).
Each record is encrypted with  [rust_sodium](https://crates.io/crates/rust_sodium).
The key is saved in the OS specific password manager provided by [keyring](https://crates.io/crates/keyring).

## Sqlite

Sqlite stores list of actions and urls. Additionally it stores configuration information.

## Full text index

Full text index is provided by the [tantivy](https://crates.io/crates/tantivy) crate.

## Json store

Initially some information was stored in a json file. The only information still provided there
is the current epic. It is not clear if this will be an useful feature in the long run. After some
more experimentation this information will either be saved in the sqlite store or deleted.
