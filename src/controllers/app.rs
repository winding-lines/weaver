use ::errors::*;
use env_logger;
use super::{flows, shell_prompt, weaver};
use super::file_utils;


/// Main dispatch function;
pub fn run() -> Result<()> {
    use ::cli::parse;
    use ::cli::Command::*;
    env_logger::init();
    let weaver = weaver::weaver_init()?;
    match parse() {
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
        MilestoneActivate(name) => {
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

