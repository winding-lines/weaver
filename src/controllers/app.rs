use ::cli;
use ::errors::*;
use ::store::Store;
use env_logger;
use super::{flows, weaver};
use super::file_utils;

/// Recommend a list of Flows for the current user.
fn recommend() -> Result<()> {
    let active = flows::active()
        .chain_err(|| "getting active flows")?;

    let mut displayed = 0;
    for name in active.iter() {
        if displayed == 0 {
            println!("recommended flows:")
        }
        displayed += 1;
        println!("  {}", name);
    }
    if displayed > 0 {
        println!("To run one use `{} run <flow-name>`", cli::APP_NAME);
    }
    Ok(())
}

/// Main dispatch function;
pub fn run() -> Result<()> {
    use ::cli::parse;
    use ::cli::Command::*;
    env_logger::init();
    let weaver = weaver::weaver_init()?;
    match parse() {
        FlowRecommend => {
            recommend()
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
        ShellPrompt => {
            println!("{}", weaver.active_milestone.as_ref()
                .map_or("<not-set>", String::as_str));
            for input in file_utils::read_stdin(1)? {
                let mut store = Store::new()?;
                store.add_shell(&input)?;
            }
            Ok(())
        }
    }
}

