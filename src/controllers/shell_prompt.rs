use ::config::file_utils;
use ::entities::Weaver;
use ::errors::*;
use ::store::Store;
use std::env;

/// Check to see if the environment is setup properly.
pub fn check() -> Result<()> {
    let ps1 = env::var("PS1").chain_err(|| "getting PS1")?;
    if !ps1.contains("weaver") {
        println!("Your PS1 variable should contain an invocation to weaver, for example");
        // note: curlies are escaped by doubling themF
        println!("export PS1='{{$( fc -ln -1 | weaver prompt)}} \\W $ '");
    } else {
        println!("Looking good");
    }
    Ok(())
}

/// Processes the actions when called from the PS1 prompt.
pub fn run(weaver: &Weaver) -> Result<()> {
    println!("{}", weaver.active_epic.as_ref()
        .map_or("<not-set>", String::as_str));
    for input in file_utils::read_stdin(1)? {
        let mut store = Store::new()?;
        store.add_shell_action(&input, weaver.active_epic.as_ref().map(String::as_str))?;
    }
    Ok(())
}