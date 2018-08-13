use super::APP_NAME;
use clap::{App, Arg, ArgGroup, ArgMatches, SubCommand};
use lib_goo::config::{OutputKind, ServerConfig};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DESCRIPTION: &str = env!["CARGO_PKG_DESCRIPTION"];

/// Commands returned by the parser for execution in the main loop.
#[derive(Debug)]
pub enum Command {
    ActionHistory(OutputKind),
    FlowRecommend,
    FlowCreate(String, bool),
    FlowRun(String),
    EpicActivate(String),
    Noop,
    ShellPrompt(bool),
}

pub struct CommandAndConfig {
    pub command: Command,
    pub server_config: ServerConfig,
}

// Constants for command names
const COMMAND_ACTIONS: &str = "actions";
const COMMAND_RUN: &str = "run";
const COMMAND_CREATE: &str = "create";
const COMMAND_EPIC: &str = "epic";
const COMMAND_PROMPT: &str = "prompt";

/// Parse a Command from the command line options.
pub fn parse() -> CommandAndConfig {
    let matches = App::new(APP_NAME)
        .version(VERSION)
        .about(DESCRIPTION)
        .arg(Arg::with_name("version")
            .short("V")
            .help("Display the version"))
        .arg(Arg::with_name("port")
            .long("port")
            .global(true)
            .takes_value(true)
            .value_name("PORT")
            .help("Select a port to connect to the server"))
        .subcommand(SubCommand::with_name(COMMAND_ACTIONS)
            .about("select one of your earlier actions")
            .arg(Arg::with_name("run")
                .long("run")
                .short("r")
                .help("run the selected action"))
            .arg(Arg::with_name("copy")
                .long("copy")
                .short("c")
                .help("copy the selected action to the clipboard"))
            .arg(Arg::with_name("print")
                .long("print")
                .short("p")
                .help("print the selected action"))
            .group(ArgGroup::with_name("output-channel")
                .args(&["run", "copy", "print"])))


        .subcommand(SubCommand::with_name(COMMAND_RUN)
            .about("run the flow with the given name")
            .arg(Arg::with_name("NAME")
                .required(true)
                .index(1)
            ))
        .subcommand(SubCommand::with_name(COMMAND_CREATE)
            .about("create a new flow with the given name, see help")
            .help("pass name as argument\n   pass on stdin any commands that should be part of the flow")
            .arg(Arg::with_name("NAME")
                .required(true)
                .index(1))
            .arg(Arg::with_name("global")
                .short("g")
                .help("create a global flow")))
        .subcommand(SubCommand::with_name(COMMAND_EPIC)
            .about("Manage epics - longer term projects/deliverables you are working on")
            .arg(Arg::with_name("NAME")
                .index(1)
                .required(true)))
        .subcommand(SubCommand::with_name(COMMAND_PROMPT)
            .about("Generate the shell prompt, call this from PS1")
            .arg(Arg::with_name("check")
                .long("check")
                .help("validate the setup")))
        .get_matches();

    let server_config = match matches.value_of("port") {
        Some(port) => ServerConfig {
            actix_address: format!("127.0.0.1:{}", port),
        },
        None => ServerConfig::current(),
    };

    CommandAndConfig {
        command: parse_command(&matches),
        server_config,
    }
}

fn parse_command(matches: &ArgMatches) -> Command {
    if matches.is_present("version") {
        println!("{}", VERSION);
        return Command::Noop;
    }
    if let Some(actions) = matches.subcommand_matches(COMMAND_ACTIONS) {
        use lib_goo::config::Channel;

        let channel = match actions.value_of("output-channel") {
            Some("run") => Channel::Run,
            Some("copy") => Channel::Copy,
            Some("print") | None => Channel::Print,
            Some(_) => panic!("bad output-channel"),
        };
        return Command::ActionHistory(OutputKind { channel });
    }
    if let Some(run) = matches.subcommand_matches(COMMAND_CREATE) {
        let name = run.value_of("NAME").unwrap();
        let global = run.is_present("global");
        return Command::FlowCreate(String::from(name), global);
    }
    if let Some(run) = matches.subcommand_matches(COMMAND_EPIC) {
        if let Some(name) = run.value_of("NAME") {
            return Command::EpicActivate(String::from(name));
        }
    }
    if let Some(prompt) = matches.subcommand_matches(COMMAND_PROMPT) {
        return Command::ShellPrompt(prompt.is_present("check"));
    }
    if let Some(run) = matches.subcommand_matches(COMMAND_RUN) {
        let name = run.value_of("NAME").unwrap();
        return Command::FlowRun(String::from(name));
    }
    Command::FlowRecommend
}
