//! Parse the command line options.
//!
use clap::{App, Arg, ArgMatches, SubCommand};
use lib_goo::config::db::PasswordSource;
use lib_goo::config::file_utils::set_app_location;
use lib_goo::config::{db, ServerConfig};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DESCRIPTION: &str = env!["CARGO_PKG_DESCRIPTION"];

pub const APP_NAME: &str = env!["CARGO_PKG_NAME"];

/// How to start the server.
#[derive(Debug)]
pub enum ServerRun {
    Foreground,
    Daemonize(Option<u32>),
}

/// Commands returned by the parser for execution in the main loop.
#[derive(Debug)]
pub enum ServerSubCommand {
    Noop,
    Start(ServerRun),
    Check,
}

pub struct CommandAndConfig {
    pub command: ServerSubCommand,
    pub server_config: ServerConfig,
    pub password_source: PasswordSource,
}

/// Parse a Command from the command line options.
pub fn parse() -> CommandAndConfig {
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
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .global(true)
                .takes_value(true)
                .value_name("PORT")
                .help("Select a port for the server to run on"),
        )
        .subcommand(SubCommand::with_name("check").about("Check to see that the server exists"))
        .subcommand(
            SubCommand::with_name("start")
                .about("Start the server")
                .arg(
                    Arg::with_name("foreground")
                        .global(true)
                        .long("fg")
                        .help("run in foreground, otherwise the default is daemon"),
                )
                .arg(
                    Arg::with_name("retries")
                        .long("retry")
                        .takes_value(true)
                        .value_name("COUNT")
                        .help("Number of times to retry the start"),
                ),
        )
        .get_matches();

    let server_config = match matches.value_of("port") {
        Some(port) => ServerConfig {
            actix_address: format!("127.0.0.1:{}", port),
        },
        None => ServerConfig::current(),
    };
    let password_source = if matches.is_present("password") {
        db::PasswordSource::Prompt
    } else {
        db::PasswordSource::Keyring
    };
    CommandAndConfig {
        command: parse_command(&matches),
        server_config,
        password_source,
    }
}

fn parse_command(matches: &ArgMatches) -> ServerSubCommand {
    if let Some(location) = matches.value_of("location") {
        set_app_location(location);
    }
    if matches.is_present("version") {
        println!("{}", VERSION);
        return ServerSubCommand::Noop;
    }
    if matches.subcommand_matches("check").is_some() {
        return ServerSubCommand::Check;
    }
    if let Some(start) = matches.subcommand_matches("start") {
        let server_run = if start.is_present("foreground") {
            ServerRun::Foreground
        } else {
            let count = start.value_of("retries").map(|c| c.parse::<u32>().unwrap());
            ServerRun::Daemonize(count)
        };
        return ServerSubCommand::Start(server_run);
    }
    unreachable!()
}
