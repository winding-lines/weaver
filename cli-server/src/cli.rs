//! Parse the command line options.
//!
use ::lib_api::config::ServerConfig;
use clap::{App, Arg, ArgMatches, SubCommand};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DESCRIPTION: &str = env!["CARGO_PKG_DESCRIPTION"];

pub const APP_NAME: &str = env!["CARGO_PKG_NAME"];


#[derive(Debug)]
pub enum ServerRun {
    Foreground,
    Daemonize,
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
}


/// Parse a Command from the command line options.
pub fn parse() -> CommandAndConfig {

    // For now the server config is hardcoded.
    let server = ServerConfig::current();

    let matches = App::new(APP_NAME)
        .version(VERSION)
        .about(DESCRIPTION)
        .arg(Arg::with_name("version")
            .short("V")
            .help("Display the version"))
        .subcommand(SubCommand::with_name("check")
            .about("Check to see that the server exists"))
        .subcommand(SubCommand::with_name("start")
            .about("Start the server")
            .arg(Arg::with_name("foreground")
                .global(true)
                .long("fg")
                .help("run in foreground, otherwise the default is daemon"))
        )
        .get_matches();

    CommandAndConfig {
        command: parse_command(&matches),
        server_config: server,
    }
}

fn parse_command(matches: &ArgMatches) -> ServerSubCommand {
    if matches.is_present("version") {
        println!("{}", VERSION);
        return ServerSubCommand::Noop;
    }
    if matches.subcommand_matches("check").is_some() {
        return ServerSubCommand::Check;
    }
    if let Some(start) = matches.subcommand_matches("start") {
        let mode = if start.is_present("foreground") {
            ServerRun::Foreground
        } else {
            ServerRun::Daemonize
        };
        return ServerSubCommand::Start(mode);
    }
    unreachable!()
}
