use ::weaver_db::config::{OutputKind, ServerRun};
use clap::{App, Arg, ArgGroup, SubCommand};
use super::APP_NAME;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const DESCRIPTION: &'static str = env!["CARGO_PKG_DESCRIPTION"];

/// Commands returned by the parser for execution in the main loop.
#[derive(Debug)]
pub enum Command {
    ActionHistory(OutputKind),
    FlowRecommend,
    FlowCreate(String, bool),
    FlowRun(String),
    EpicActivate(String),
    EpicList,
    Noop,
    Server(ServerSubCommand),
    ShellPrompt(bool),
    Data(DataSubCommand),
}


/// Data subcommands.
#[derive(Debug)]
pub enum DataSubCommand {
    Sqlite,
}

/// Server subcommands
#[derive(Debug)]
pub enum ServerSubCommand {
    Start(ServerRun),
    Check,
}

// Constants for command names
const COMMAND_ACTIONS: &str = "actions";
const COMMAND_RUN: &str = "run";
const COMMAND_CREATE: &str = "create";
const COMMAND_EPIC: &str = "epic";
const COMMAND_PROMPT: &str = "prompt";
const COMMAND_SERVER: &str = "server";
const COMMAND_DATA: &str = "data";

/// Parse a Command from the command line options.
pub fn parse() -> Command {
    let matches = App::new(APP_NAME)
        .version(VERSION)
        .about(DESCRIPTION)
        .arg(Arg::with_name("version")
            .short("V")
            .help("Display the version"))
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
            .arg(Arg::with_name("path")
                .long("op")
                .help("output the path"))
            .arg(Arg::with_name("command")
                .long("oc")
                .help("output the command"))
            .arg(Arg::with_name("path-with-command")
                .long("opc")
                .help(" output the path and the command"))
            .group(ArgGroup::with_name("output-channel")
                .args(&["run", "copy", "print"]))
            .group(ArgGroup::with_name("output-content")
                .args(&["path", "command", "path-with-command"])))
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
                .index(1))
            .arg(Arg::with_name("list")
                .short("l")
                .help("list current epics")))
        .subcommand(SubCommand::with_name(COMMAND_PROMPT)
            .about("Generate the shell prompt, call this from PS1")
            .arg(Arg::with_name("check")
                .long("check")
                .help("validate the setup")))
        .subcommand(SubCommand::with_name(COMMAND_SERVER)
            .about("Interact with the embedded API server")
            .subcommand(SubCommand::with_name("check")
                .about("Check to see that the server exists"))
            .subcommand(SubCommand::with_name("start")
                .about("Start the server")
                .arg(Arg::with_name("foreground")
                    .global(true)
                    .long("fg")
                    .help("run in foreground, otherwise the default is daemon"))
            ))
        .subcommand(SubCommand::with_name(COMMAND_DATA)
            .about("Manipulate the stored data")
            .subcommand(SubCommand::with_name("sqlite")
                .about("Start an sqlite3 shell")))
        .get_matches();

    if matches.is_present("version") {
        println!("{}", VERSION);
        return Command::Noop;
    }
    if let Some(_actions) = matches.subcommand_matches(COMMAND_ACTIONS) {
        use weaver_db::config::{Content, Channel};

        let content = match _actions.value_of("output-content") {
            Some("path") => Content::Path,
            Some("path-with-command") => Content::PathWithCommand,
            Some("command") | None => Content::Command,
            Some(_) => panic!("bad output-content"),
        };
        let channel = match _actions.value_of("output-channel") {
            Some("run") => Channel::Run,
            Some("copy") => Channel::Copy,
            Some("print") | None => Channel::Print,
            Some(_) => panic!("bad output-channel"),
        };
        return Command::ActionHistory(OutputKind { content, channel });
    }
    if let Some(run) = matches.subcommand_matches(COMMAND_CREATE) {
        let name = run.value_of("NAME").unwrap();
        let global = run.is_present("global");
        return Command::FlowCreate(String::from(name), global);
    }
    if let Some(run) = matches.subcommand_matches(COMMAND_EPIC) {
        if let Some(name) = run.value_of("NAME") {
            return Command::EpicActivate(String::from(name));
        } else {
            return Command::EpicList;
        }
    }
    if let Some(prompt) = matches.subcommand_matches(COMMAND_PROMPT) {
        return Command::ShellPrompt(prompt.is_present("check"));
    }
    if let Some(run) = matches.subcommand_matches(COMMAND_RUN) {
        let name = run.value_of("NAME").unwrap();
        return Command::FlowRun(String::from(name));
    }
    if let Some(run) = matches.subcommand_matches(COMMAND_SERVER) {
        if run.subcommand_matches("check").is_some() {
            return Command::Server(ServerSubCommand::Check);
        }
        if let Some(start) = run.subcommand_matches("start") {
            let mode = if start.is_present("foreground") {
                ServerRun::Foreground
            } else {
                ServerRun::Daemonize
            };
            return Command::Server(ServerSubCommand::Start(mode));
        }
        unreachable!()
    }
    if let Some(run) = matches.subcommand_matches(COMMAND_DATA) {
        if run.subcommand_matches("sqlite").is_some() {
            return Command::Data(DataSubCommand::Sqlite);
        }
    }
    Command::FlowRecommend
}
