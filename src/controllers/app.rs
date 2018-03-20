use ::cli::Command::*;
use ::cli::parse;
use ::config::file_utils;
use ::errors::*;
use display;
use server;
use store::{actions, Store};
use super::{flows, shell_prompt, shell_proxy};
use ::config::ServerRun;


/// Main dispatch function;
pub fn run() -> Result<()> {
    let mut store = Store::new()?;
    let wanted = parse();
    debug!("Executing cli command {:?}", wanted);
    match wanted {
        ActionHistory => {
            let epic = store.epic()?;
            let actions = actions::history(&mut store, &epic.as_ref().map(String::as_str))?;
            if let Some(selection) = display::show(actions)? {
                if selection.kind == "shell" {
                    shell_proxy::run(selection.name)
                        .map(|_| ())
                } else {
                    shell_proxy::run(format!("open {}", selection.name))
                        .map(|_| ())
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

