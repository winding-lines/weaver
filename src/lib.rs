//! Weaver is a history tracker for you command line and shell commands.
//!
//! The long term goal is to help you improve your productivity by relying on
//! locally stored data. This should provide better privacy and allow you to combine
//! work and public domain information in one integrated view.
//!
//! The command line interface tools are implemented in the following crates:
//!
//! * [cli-shell](../cli_shell/index.html): integration in your shell flows
//! * [cli-server](../cli_server/index.html): contains the server which manages the information
//! * [cli-replay](../cli_replay/index.html): allow you to replay your encrypted documents to recreate the other stores
//!
//! Weaver provides library through of workspace crates.
//!
//! * [weaver-db](../weaver_db/index.html): holds the local database for actions
//! * [weaver-error](../weaver_error/index.html): error structs used by all crates
//! * [weaver-index](../weaver_index/index.html): full text search
//! * [weaver-rpc](../weaver_rpc/index.html): client implementation of API, used by the cli
//! * [weaver-web](../weaver_web/index.html): API and base page server
