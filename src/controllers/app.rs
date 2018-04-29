use ::cli::Command::*;
use ::cli::parse;
use ::cli::{CommandAndConfig, ServerConfig, ServerSubCommand};
use clipboard::{ClipboardContext, ClipboardProvider};
use display;
use std::sync::Arc;
use super::{data, flows, server, shell_prompt, shell_proxy};
use weaver_db::{local_api, RealStore};
use weaver_db::config::{file_utils, OutputKind, ServerRun};
use weaver_error::*;
use weaver_rpc;

fn run_history(store: &RealStore, config: &ServerConfig, output_kind: OutputKind, grpc: bool) -> Result<()> {
    use weaver_db::config::Channel::*;

    let epic = store.epic()?;
    let actions = if grpc {
        weaver_rpc::client::history(epic, &config.rpc_address)?
    } else {
        local_api::history(epic, &store.connection()? )?
    };
    let user_selection = display::show(actions, output_kind)?;
    if let Some(action) = user_selection.action {
        match user_selection.kind {
            Some(OutputKind { channel: Run, content: ref e }) => {
                if action.kind == "shell" {
                    shell_proxy::run(action.as_shell_command(e))
                        .map(|_| ())
                } else {
                    shell_proxy::run(format!("open {}", action.name))
                        .map(|_| ())
                }
            }
            Some(OutputKind { channel: Copy, content: ref e }) => {
                eprintln!("Copying to clipboard: {}", action.name);
                if let Ok(mut ctx) = ClipboardContext::new() {
                    ctx.set_contents(action.as_shell_command(e)).expect("set clipboard");
                }
                Ok(())
            }
            Some(OutputKind { channel: Print, content: ref e }) => {
                println!("{}", action.as_shell_command(e));
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
    let store = Arc::new(RealStore::new()?);
    let CommandAndConfig { command, server: server_config} = parse();
    debug!("Executing cli command {:?}", command);
    match command {
        ActionHistory(output_kind, grpc) => run_history(&*store, &server_config, output_kind, grpc),
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
            store.epic_names().map(|names| {
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
                let maybe_epic = store.epic()?;
                if store.weaver().start_server.unwrap_or(false) && !server::is_running(&server_config) {
                    let my_store = Arc::clone(&store);
                    let _ = server::start(&ServerRun::Daemonize, &server_config, my_store)?;
                }
                shell_prompt::run(&store, maybe_epic.as_ref().map(String::as_str))
            }
        }
    }
}

