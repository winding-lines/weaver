use ::errors::*;
use env_logger;
use super::{flows, shell_prompt, weaver};
use super::file_utils;
use store::{Store, actions};


/// Main dispatch function;
pub fn run() -> Result<()> {
    use ::cli::parse;
    use ::cli::Command::*;
    env_logger::init();
    let weaver = weaver::weaver_init()?;
    let mut store = Store::new()?;
    match parse() {
        ActionHistory => {
            let epic = weaver.active_epic.as_ref().map(String::as_str);
            let actions = actions::history(&mut store, epic)?;
            println!("{:?}", actions);
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
            weaver::milestone_activate(name)
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

