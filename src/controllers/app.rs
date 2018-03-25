use ::cli::Command::*;
use ::cli::parse;
use ::config::{OutputKind, file_utils, ServerRun};
use ::errors::*;
use display;
use server;
use store::{actions, Store};
use super::{flows, shell_prompt, shell_proxy};


/// Main dispatch function;
pub fn run() -> Result<()> {
    let mut store = Store::new()?;
    let wanted = parse();
    debug!("Executing cli command {:?}", wanted);
    match wanted {
        ActionHistory(kind) => {
            let epic = store.epic()?;
            let actions = actions::history(&mut store, &epic.as_ref().map(String::as_str))?;
            let user_selection = display::show(actions, kind)?;
            if let Some(action) = user_selection.action {
                match user_selection.kind {
                    Some(OutputKind::Run) => {
                        if action.kind == "shell" {
                            shell_proxy::run(action.name)
                                .map(|_| ())
                        } else {
                            shell_proxy::run(format!("open {}", action.name))
                                .map(|_| ())
                        }
                    }
                    Some(OutputKind::Copy) => {
                        use clipboard::*;
                        if let Ok(mut ctx) = ClipboardContext::new() {
                            ctx.set_contents(action.name).expect("set clipboard");
                        }
                        Ok(())
                    }
                    Some(OutputKind::Print) => {
                        if action.kind == "shell" {
                            print!("{}", action.name);
                        } else {
                            print!("open {}", action.name);
                        }
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
            store.set_epic(name)
        }
        Noop => {
            Ok(())
        }
        Server(ref run) => {
            server::start(run).map(|_| ())
        }
        ShellPrompt(check) => {
            if check {
                shell_prompt::check()
            } else {
                let maybe_epic = store.epic()?;
                if store.weaver().start_server.unwrap_or(false) && !server::is_running() {
                    let _ = server::start(&ServerRun::Daemonize)?;
                }
                shell_prompt::run(maybe_epic.as_ref().map(String::as_str))
            }
        }
    }
}

