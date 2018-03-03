use ::cli::Command::*;
use ::cli::parse;
use ::config::file_utils;
use ::errors::*;
use display;
use store::{actions, Store};
use super::{flows, shell_prompt, weaver};


/// Main dispatch function;
pub fn run() -> Result<()> {
    let weaver = weaver::weaver_init()?;
    debug!("weaver initialized, active_epic {:?}", weaver.active_epic);
    let mut store = Store::new()?;
    match parse() {
        ActionHistory => {
            let epic = weaver.active_epic.as_ref().map(String::as_str);
            let actions = actions::history(&mut store, epic)?;
            let selected = display::show(actions)?;
            println!("selected {:?}", selected);
            Ok(())
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
        ShellPrompt(check) => {
            if check {
                shell_prompt::check()
            } else {
                shell_prompt::run(&weaver)
            }
        }
    }
}

