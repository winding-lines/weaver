use ::cli::Command::*;
use ::cli::parse;
use ::config::file_utils;
use ::errors::*;
use display;
use server;
use store::{actions, Store};
use super::{flows, shell_prompt, shell_proxy, weaver};


/// Main dispatch function;
pub fn run() -> Result<()> {
    let weaver = weaver::weaver_init()?;
    debug!("weaver initialized, active_epic {:?}", weaver.active_epic);
    let mut store = Store::new()?;
    let wanted = parse();
    debug!("Executing cli command {:?}", wanted);
    match wanted {
        ActionHistory => {
            let epic = weaver.active_epic.as_ref().map(String::as_str);
            let actions = actions::history(&mut store, epic)?;
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
            weaver::epic_activate(name)
        }
        Noop => {
            Ok(())
        }
        Server => {
            server::start().map(|_| ())
        }
        ShellPrompt(check) => {
            if check {
                shell_prompt::check()
            } else {
                shell_prompt::run(&weaver)
            }
        }
    }
}

