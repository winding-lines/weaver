//! Weaver is a history tracker for you command line and shell commands.
//!
//! The long term goal is to help you improve your productivity by relying on
//! locally stored data. This should provide better privacy and allow you to combine
//! work and public domain information in one integrated view.
//!
//! The command line interface tools are implemented in the following crates:
//!
//! * [weaver](../weaver/index.html): integration in your shell flows
//! * [weaver-server](../weaver_server/index.html): contains the server which manages the information
//! * [weaver-data](../weaver-data/index.html): manages the various stores
//!
//! Weaver provides library through of workspace crates.
//!
//! * [lib-db](../lib_db/index.html): holds the local database for actions
//! * [lib-error](../lib_error/index.html): error structs used by all crates
//! * [lib-index](../lib_index/index.html): full text search
//! * [lib-rpc](../lib_rpc/index.html): client implementation of API, used by the cli
//! * [lib-server](../lib_server/index.html): API and base page server
