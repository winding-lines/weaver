use ::cli::Command::*;
use ::cli::parse;
use ::cli::{CommandAndConfig, ServerSubCommand};
use std::sync::Arc;
use super::{data, flows, history, server, shell_prompt};
use local_api;
use weaver_db::RealStore;
use weaver_db::config::{file_utils, ServerRun, Environment};
use weaver_error::*;


/// Main dispatch function;
pub fn run() -> Result<()> {
    let CommandAndConfig { command, server_config, api_config} = parse();
    let store = Arc::new(RealStore::new(api_config)?);
    debug!("Executing cli command {:?}", command);
    let epic = store.epic()?;
    let env = Arc::new(Environment::build(epic)?);
    match command {
        ActionHistory(output_kind) => history::run( Arc::clone(&store), &output_kind, &env),
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
        Data(sub) => {
            data::run(&*store, &sub)
        }
        EpicActivate(name) => {
            RealStore::save_epic(name)
        }
        EpicList => {
            local_api::epic_names(&store.destination()).map(|names| {
                for n in names {
                    println!("{}", n)
                }
            })
        }
        Noop => {
            Ok(())
        }
        Server(ServerSubCommand::Start(ref mode)) => {
            server::start(mode, &server_config, store).map(|_| ())
        }
        Server(ServerSubCommand::Check) => {
            server::check(&server_config)
        }
        ShellPrompt(check) => {
            if check {
                shell_prompt::check()
            } else {
                if store.weaver().start_server.unwrap_or(false) && !server::is_running(&server_config) {
                    let my_store = Arc::clone(&store);
                    let _ = server::start(&ServerRun::Daemonize, &server_config, my_store)?;
                }
                // Run the shell prompt, we do not want too many errors here.
                shell_prompt::run(&store, &env)
                    .or_else(|_| Ok(()))
            }
        }
    }
}

