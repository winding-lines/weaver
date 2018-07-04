use ::cli::Command::*;
use ::cli::CommandAndConfig;
use ::cli::parse;
use lib_api::config::{Destination, Environment, file_utils};
use local_api;
use local_store::epics;
use std::sync::Arc;
use super::{flows, history, shell_prompt};
use lib_error::*;


/// Main dispatch function;
pub fn run() -> Result<()> {
    let CommandAndConfig { command, server_config } = parse();
    let destination = Destination::Remote(server_config.actix_address);
    debug!("Executing cli command {:?}", command);
    let epic = epics::epic()?;
    let env = Arc::new(Environment::build(epic)?);
    match command {
        ActionHistory(output_kind) => history::run(&destination, &output_kind, &env),
        FlowRecommend => {
            flows::recommend()
        }
        FlowRun(name) => {
            flows::run(name)
        }
        FlowCreate(name, global) => {
            let actions = file_utils::read_stdin(50)?;
            flows::create(name, global, actions)
        }
        EpicActivate(name) => {
            epics::save_epic(name)
        }
        EpicList => {
            local_api::epic_names(&destination).map(|names| {
                for n in names {
                    println!("{}", n.name)
                }
            })
        }
        Noop => {
            Ok(())
        }
        ShellPrompt(check) => {
            if check {
                shell_prompt::check()
            } else {
                // Run the shell prompt, we do not want too many errors here.
                shell_prompt::run(&destination, &env)
                    .or_else(|_| Ok(()))
            }
        }
    }
}

