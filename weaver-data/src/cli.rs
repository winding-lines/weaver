use clap::{App, Arg, SubCommand};
use lib_goo::config::db;
use lib_goo::config::file_utils::set_app_location;
use lib_index::repo::Collection;

pub const APP_NAME: &str = env!["CARGO_PKG_NAME"];
const VERSION: &str = env!("CARGO_PKG_VERSION");
const DESCRIPTION: &str = env!["CARGO_PKG_DESCRIPTION"];

/// Commands returned by the parser for execution in the main loop.
#[derive(Debug)]
pub enum DataSubCommand {
    /// Backup the current database, to be used before running a migration, for example.
    Backup,
    /// Create the various stores.
    Create,
    /// Check the various stores.
    Check,
    /// Decrypt the store document with the given hash (= filename under the repo folder)
    Decrypt(Collection, String),
    /// Dump the content of the url policies (restrictions) table.
    DumpUrlPolicies,
    /// Encrypt the file with the given name and save in the repo.
    Encrypt(Collection, String),
    Noop,
    /// Delete the text index and rebuilds it by replaying the document in the store.
    RebuildIndex,
    /// Link the commands and pages tables.
    LinkCommandPages,
    /// Run the sqlite shell on the weaver db
    Sqlite,
}

pub struct ConfigAndCommand {
    pub password_source: Option<db::PasswordSource>,
    pub command: DataSubCommand,
}

/// Parse a Command and Configuration properties from the command line options.
pub fn parse() -> ConfigAndCommand {
    let matches = App::new(APP_NAME)
        .version(VERSION)
        .about(DESCRIPTION)
        .arg(
            Arg::with_name("version")
                .short("V")
                .help("Display the version"),
        )
        .arg(
            Arg::with_name("location")
                .short("C")
                .long("location")
                .takes_value(true)
                .value_name("FOLDER")
                .help("Select a different location for the store"),
        )
        .arg(
            Arg::with_name("password")
                .short("P")
                .long("password")
                .help("Prompt for password - instead of the default of taking from keyring"),
        )
        .subcommand(
            SubCommand::with_name("backup").about("Create a backup of the existing database"),
        )
        .subcommand(SubCommand::with_name("sqlite").about("Start an sqlite3 shell"))
        .subcommand(SubCommand::with_name("setup").about("Create the sqlite3 database"))
        .subcommand(
            SubCommand::with_name("encrypt")
                .arg(
                    Arg::with_name("collection")
                        .long("collection")
                        .required(true)
                        .takes_value(true)
                        .value_name("COLLECTION")
                        .help("Specify the repo collection to operate on"),
                )
                .arg(Arg::with_name("NAME").index(1))
                .about("Encrypt a file"),
        )
        .subcommand(
            SubCommand::with_name("decrypt")
                .arg(
                    Arg::with_name("collection")
                        .long("collection")
                        .required(true)
                        .takes_value(true)
                        .value_name("COLLECTION")
                        .help("Specify the repo collection to operate on"),
                )
                .arg(Arg::with_name("NAME").index(1))
                .about("Decrypt the handle"),
        )
        .subcommand(SubCommand::with_name("check").about("Validate the state of the various repos"))
        .subcommand(
            SubCommand::with_name("rebuild-index")
                .about("Rebuild the text search index from the files in the encrypted repo"),
        )
        .subcommand(
            SubCommand::with_name("link-commands-pages")
                .about("Link the commands page_id with the pages id, when matching"),
        )
        .subcommand(
            SubCommand::with_name("dump-url-policies").about("Show the current url policies"),
        )
        .get_matches();

    if let Some(location) = matches.value_of("location") {
        set_app_location(location);
    }
    if matches.is_present("version") {
        println!("{}", VERSION);
        return ConfigAndCommand {
            password_source: None,
            command: DataSubCommand::Noop,
        };
    }
    let password_source = if matches.is_present("password") {
        db::PasswordSource::Prompt
    } else {
        db::PasswordSource::Keyring
    };
    let command = if matches.subcommand_matches("backup").is_some() {
        DataSubCommand::Backup
    } else if matches.subcommand_matches("sqlite").is_some() {
        DataSubCommand::Sqlite
    } else if matches.subcommand_matches("setup").is_some() {
        DataSubCommand::Create
    } else if matches.subcommand_matches("check").is_some() {
        DataSubCommand::Check
    } else if let Some(encrypt) = matches.subcommand_matches("encrypt") {
        let name = encrypt.value_of("NAME").unwrap();
        let collection = encrypt.value_of("collection").unwrap();
        DataSubCommand::Encrypt(Collection(collection.into()), name.to_string())
    } else if let Some(decrypt) = matches.subcommand_matches("decrypt") {
        let name = decrypt.value_of("NAME").unwrap();
        let collection = decrypt.value_of("collection").unwrap();
        DataSubCommand::Decrypt(Collection(collection.into()), name.to_string())
    } else if matches.subcommand_matches("rebuild-index").is_some() {
        DataSubCommand::RebuildIndex
    } else if matches.subcommand_matches("link-commands-pages").is_some() {
        DataSubCommand::LinkCommandPages
    } else if matches.subcommand_matches("dump-url-policies").is_some() {
        DataSubCommand::DumpUrlPolicies
    } else {
        unreachable!()
    };
    ConfigAndCommand {
        password_source: Some(password_source),
        command,
    }
}
