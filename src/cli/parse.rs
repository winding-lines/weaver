use clap::{App, Arg, SubCommand};
use super::APP_NAME;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const DESCRIPTION: &'static str = env!["CARGO_PKG_DESCRIPTION"];

/// Commands returned by the parser for execution in the main loop.
pub enum Command {
    FlowRecommend,
    FlowCreate(String, bool),
    FlowRun(String),
    MilestoneActivate(String),
    Noop,
    ShellPrompt,
}

/// Parse a Command from the command line options.
pub fn parse() -> Command {
    let matches = App::new(APP_NAME)
        .version(VERSION)
        .about(DESCRIPTION)
        .arg(Arg::with_name("version")
            .short("V")
            .help("Display the version"))
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
        .subcommand(SubCommand::with_name("prompt")
            .about("Generate the shell prompt, call this from PS1"))
        .subcommand(SubCommand::with_name("milestone")
            .about("Set the active milestone that you are working towards")
            .arg(Arg::with_name("NAME")
                .required(true)
                .index(1)
            ))
        .get_matches();

    if matches.is_present("version") {
        println!("{}", VERSION);
        return Command::Noop;
    }
    if let Some(run) = matches.subcommand_matches("run") {
        let name = run.value_of("NAME").unwrap();
        return Command::FlowRun(String::from(name));
    }
    if let Some(run) = matches.subcommand_matches("create") {
        let name = run.value_of("NAME").unwrap();
        let global = run.is_present("global");
        return Command::FlowCreate(String::from(name), global);
    }
    if let Some(run) = matches.subcommand_matches("milestone") {
        let name = run.value_of("NAME").unwrap();
        return Command::MilestoneActivate(String::from(name));
    }
    if matches.subcommand_matches("prompt").is_some() {
        return Command::ShellPrompt;
    }

    Command::FlowRecommend
}
