use ::cli::Command::*;
use ::cli::parse;
use ::cli::ServerSubCommand;
use clipboard::{ClipboardContext, ClipboardProvider};
use display;
use std::sync::Arc;
use super::{data, flows, server, shell_prompt, shell_proxy};
use weaver_db::{actions, RealStore};
use weaver_db::config::{file_utils, OutputKind, ServerRun};
use weaver_error::*;

fn run_history(store: &RealStore, output_kind: OutputKind) -> Result<()> {
    use weaver_db::config::Channel::*;

    let epic = store.epic()?;
    let actions = actions::history(&store.connection()?, &epic.as_ref().map(String::as_str))?;
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
    let wanted = parse();
    debug!("Executing cli command {:?}", wanted);
    match wanted {
        ActionHistory(output_kind) => run_history(&*store, output_kind),
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
            server::start(mode, store).map(|_| ())
        }
        Server(ServerSubCommand::Check) => {
            server::check()
        }
        ShellPrompt(check) => {
            if check {
                shell_prompt::check()
            } else {
                let maybe_epic = store.epic()?;
                if store.weaver().start_server.unwrap_or(false) && !server::is_running() {
                    let my_store = Arc::clone(&store);
                    let _ = server::start(&ServerRun::Daemonize, my_store)?;
                }
                shell_prompt::run(& store, maybe_epic.as_ref().map(String::as_str))
            }
        }
    }
}

