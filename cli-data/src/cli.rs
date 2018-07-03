use clap::{App, Arg, SubCommand};

pub const APP_NAME: &str = env!["CARGO_PKG_NAME"];
const VERSION: &str = env!("CARGO_PKG_VERSION");
const DESCRIPTION: &str = env!["CARGO_PKG_DESCRIPTION"];

/// Commands returned by the parser for execution in the main loop.
#[derive(Debug)]
pub enum DataSubCommand {
    Noop,
    Sqlite,
    Create,
    Encrypt(String),
    Decrypt(String),
    Check
}

/// Parse a Command from the command line options.
pub fn parse() -> DataSubCommand {
    let matches = App::new(APP_NAME)
        .version(VERSION)
        .about(DESCRIPTION)
        .arg(Arg::with_name("version")
            .short("V")
            .help("Display the version"))
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
        .get_matches();

    if matches.is_present("version") {
        println!("{}", VERSION);
        return DataSubCommand::Noop;
    }
    if matches.subcommand_matches("sqlite").is_some() {
        return DataSubCommand::Sqlite;
    }
    if matches.subcommand_matches("setup").is_some() {
        return DataSubCommand::Create;
    }
    if matches.subcommand_matches("check").is_some() {
        return DataSubCommand::Check;
    }
    if let Some(encrypt) = matches.subcommand_matches("encrypt") {
        let name = encrypt.value_of("NAME").unwrap();
        return DataSubCommand::Encrypt(name.to_string());
    }
    if let Some(decrypt) = matches.subcommand_matches("decrypt") {
        let name = decrypt.value_of("NAME").unwrap();
        return DataSubCommand::Decrypt(name.to_string());
    }
    unreachable!()
}
