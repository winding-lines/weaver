use ::config::{OutputKind, ServerRun};
use clap::{App, Arg, SubCommand};
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
    Noop,
    Server(ServerRun),
    ShellPrompt(bool),
}

/// Parse a Command from the command line options.
pub fn parse() -> Command {
    let matches = App::new(APP_NAME)
        .version(VERSION)
        .about(DESCRIPTION)
        .arg(Arg::with_name("version")
            .short("V")
            .help("Display the version"))
        .subcommand(SubCommand::with_name("actions")
            .about("pick an action")
            .arg(Arg::with_name("run")
                .short("r")
                .help("run the selected action"))
            .arg(Arg::with_name("copy")
                .short("c")
                .help("copy the selected action to the clipboard"))
            .arg(Arg::with_name("copy-with-context")
                .short("w")
                .help("copy the selected action and its context (working dir) to the clipboard")))
        .subcommand(SubCommand::with_name("run")
            .about("run the flow with the given name")
            .arg(Arg::with_name("NAME")
                .required(true)
                .index(1)
            ))
        .subcommand(SubCommand::with_name("create")
            .about("create a new flow with the given name, see help")
            .help("pass name as argument\n   pass on stdin any commands that should be part of the flow")
            .arg(Arg::with_name("NAME")
                .required(true)
                .index(1))
            .arg(Arg::with_name("global")
                .short("g")
                .help("create a global flow")))
        .subcommand(SubCommand::with_name("epic")
            .about("Set the current epic (task, story, project) that you are working towards")
            .arg(Arg::with_name("NAME")
                .required(true)
                .index(1)))
        .subcommand(SubCommand::with_name("prompt")
            .about("Generate the shell prompt, call this from PS1")
            .arg(Arg::with_name("check")
                .long("check")
                .help("validate the setup")))
        .subcommand(SubCommand::with_name("server")
            .about("Start the embedded API server")
            .arg(Arg::with_name("foreground")
                .long("fg")
                .help("run in foreground, otherwise the default is daemon")))
        .get_matches();

    if matches.is_present("version") {
        println!("{}", VERSION);
        return Command::Noop;
    }
    if let Some(_actions) = matches.subcommand_matches("actions") {
        let kind = if _actions.is_present("copy-with-context") {
            OutputKind::CopyWithContext
        } else if _actions.is_present("copy") {
            OutputKind::Copy
        } else {
            OutputKind::Run
        };
        return Command::ActionHistory(kind);
    }
    if let Some(run) = matches.subcommand_matches("create") {
        let name = run.value_of("NAME").unwrap();
        let global = run.is_present("global");
        return Command::FlowCreate(String::from(name), global);
    }
    if let Some(run) = matches.subcommand_matches("epic") {
        let name = run.value_of("NAME").unwrap();
        return Command::EpicActivate(String::from(name));
    }
    if let Some(prompt) = matches.subcommand_matches("prompt") {
        return Command::ShellPrompt(prompt.is_present("check"));
    }
    if let Some(run) = matches.subcommand_matches("run") {
        let name = run.value_of("NAME").unwrap();
        return Command::FlowRun(String::from(name));
    }
    if let Some(run) = matches.subcommand_matches("server") {
        let mode = if run.is_present("foreground") {
            ServerRun::Foreground
        } else {
            ServerRun::Daemonize
        };
        return Command::Server(mode);
    }
    Command::FlowRecommend
}
