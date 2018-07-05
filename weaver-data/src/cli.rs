use clap::{App, Arg, SubCommand};
use lib_api::config::db;
use lib_api::config::file_utils::set_app_location;

pub const APP_NAME: &str = env!["CARGO_PKG_NAME"];
const VERSION: &str = env!("CARGO_PKG_VERSION");
const DESCRIPTION: &str = env!["CARGO_PKG_DESCRIPTION"];

/// Commands returned by the parser for execution in the main loop.
#[derive(Debug)]
pub enum DataSubCommand {
    /// Create the various stores
    Create,
    /// Check the various stores.
    Check,
    /// Decrypt the store document with the given hash (= filename under the repo folder)
    Decrypt(String),
    /// Encrypt the file with the given name and save in the repo.
    Encrypt(String),
    Noop,
    /// Delete the text index and rebuilds it by replaying the document in the store.
    RebuildIndex,
    /// Run the sqlite shell on the weaver db
    Sqlite,
}

pub struct ConfigAndCommand {
    pub password_source: Option<db::PasswordSource>,
    pub command: DataSubCommand,
}

/// Parse a Command from the command line options.
pub fn parse() -> ConfigAndCommand {
    let matches = App::new(APP_NAME)
        .version(VERSION)
        .about(DESCRIPTION)
        .arg(Arg::with_name("version")
            .short("V")
            .help("Display the version"))
        .arg(Arg::with_name("location")
            .short("C")
            .long("location")
            .takes_value(true)
            .value_name("FOLDER")
            .help("Select a different location for the store"))
        .arg(Arg::with_name("password")
            .short("P")
            .long("password")
            .help("Prompt for password - instead of the default of taking from keyring"))
        .subcommand(SubCommand::with_name("sqlite")
            .about("Start an sqlite3 shell"))
        .subcommand(SubCommand::with_name("setup")
            .about("Create the sqlite3 database"))
        .subcommand(SubCommand::with_name("encrypt")
            .arg(Arg::with_name("NAME")
                .index(1))
            .about("Encrypt a file"))
        .subcommand(SubCommand::with_name("decrypt")
            .arg(Arg::with_name("NAME")
                .index(1))
            .about("Decrypt the handle"))
        .subcommand(SubCommand::with_name("check")
            .about("Validate the state of the various repos"))
        .subcommand(SubCommand::with_name("rebuild-index")
            .about("Rebuild the text search index from the files in the encrypted repo"))
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
    let command = if matches.subcommand_matches("sqlite").is_some() {
        DataSubCommand::Sqlite
    } else if matches.subcommand_matches("setup").is_some() {
        DataSubCommand::Create
    } else if matches.subcommand_matches("check").is_some() {
        DataSubCommand::Check
    } else if let Some(encrypt) = matches.subcommand_matches("encrypt") {
        let name = encrypt.value_of("NAME").unwrap();
        DataSubCommand::Encrypt(name.to_string())
    } else if let Some(decrypt) = matches.subcommand_matches("decrypt") {
        let name = decrypt.value_of("NAME").unwrap();
        DataSubCommand::Decrypt(name.to_string())
    } else if matches.subcommand_matches("rebuild-index").is_some() {
        DataSubCommand::RebuildIndex
    } else {
        unreachable!()
    };
    ConfigAndCommand {
        password_source: Some(password_source),
        command
    }
}