use ::cli::Command::*;
use ::cli::parse;
use ::cli::{CommandAndConfig, ServerSubCommand};
use clipboard::{ClipboardContext, ClipboardProvider};
use display;
use std::sync::Arc;
use super::{data, flows, server, shell_prompt, shell_proxy};
use local_api;
use weaver_db::{Destination, RealStore};
use weaver_db::config::{file_utils, OutputKind, ServerRun, Environment};
use weaver_error::*;

fn run_history(destination: &Destination, output_kind: OutputKind, env: Arc<Environment>) -> Result<()> {
    use weaver_db::config::Channel::*;

    let actions = local_api::history(&env, &destination)?;
    let user_selection = display::show(actions, output_kind, Arc::clone(&env))?;
    if let Some(action) = user_selection.action {
        match user_selection.kind {
            Some(OutputKind { channel: Run, ref content }) => {
                if action.kind == "shell" {
                    shell_proxy::run(action.to_shell_command(content, &env))
                        .map(|_| ())
                } else {
                    shell_proxy::run(format!("open {}", action.name))
                        .map(|_| ())
                }
            }
            Some(OutputKind { channel: Copy, ref content }) => {
                eprintln!("Copying to clipboard: {}", action.name);
                if let Ok(mut ctx) = ClipboardContext::new() {
                    ctx.set_contents(action.to_shell_command(content, &env)).expect("set clipboard");
                }
                Ok(())
            }
            Some(OutputKind { channel: Print, ref content }) => {
                println!("{}", action.to_shell_command(content, &env));
                Ok(())
            }
            None => {
                eprintln!("No action kind passed in");
                Ok(())
            }
        }
    } else {
        eprintln!("No command selected from history");
        Ok(())
    }
}


/// Main dispatch function;
pub fn run() -> Result<()> {
    let CommandAndConfig { command, server_config, api_config} = parse();
    let store = Arc::new(RealStore::new(api_config)?);
    debug!("Executing cli command {:?}", command);
    let epic = store.epic()?;
    let env = Arc::new(Environment::build(epic)?);
    match command {
        ActionHistory(output_kind) => run_history( &store.destination(), output_kind, env),
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
            data::run(&*store, sub)
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
                shell_prompt::run(&store, &env)
            }
        }
    }
}

