use super::{flows, history, shell_prompt};
use cli::parse;
use cli::Command::*;
use cli::CommandAndConfig;
use lib_error::*;
use lib_goo::config::{file_utils, Destination, Environment};
use local_store::epics;
use std::sync::Arc;

/// Main dispatch function;
pub fn run() -> Result<()> {
    let CommandAndConfig {
        command,
        server_config,
    } = parse();
    let destination = Destination::Remote(server_config.actix_address());
    debug!("Executing cli command {:?}", command);
    let epic = epics::epic()?;
    let env = Arc::new(Environment::build(epic)?);
    match command {
        ActionHistory(output_kind) => history::run(&destination, &output_kind, &env),
        FlowRecommend => flows::recommend(),
        FlowRun(name) => flows::run(name),
        FlowCreate(name, global) => {
            let actions = file_utils::read_stdin(50)?;
            flows::create(name, global, actions)
        }
        EpicActivate(name) => epics::save_epic(name),
        Noop => Ok(()),
        ShellPrompt(check) => {
            if check {
                shell_prompt::check()
            } else {
                // Run the shell prompt, we do not want too many errors here.
                shell_prompt::run(&destination, &env).or_else(|_| Ok(()))
            }
        }
    }
}
