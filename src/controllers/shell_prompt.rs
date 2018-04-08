use ::config::file_utils;
use ::errors::*;
use ::store::RealStore;
use ::store::actions;
use std::env;

/// Check to see if the environment is setup properly.
pub fn check() -> Result<()> {
    let ps1 = env::var("PS1").chain_err(|| "getting PS1")?;
    if !ps1.contains("weaver") {
        println!("Your PS1 variable should contain an invocation to weaver, for example");
        // note: curlies are escaped by doubling them
        println!("export PS1='{{$( fc -ln -1 | weaver prompt)}} \\W $ '");
    } else {
        println!("Looking good");
    }
    Ok(())
}

/// Processes the actions when called from the PS1 prompt.
pub fn run(mut store: &mut RealStore, epic: Option<&str>) -> Result<()> {


    // output the current epic so that it can end up in the prompt
    println!("{}", epic.unwrap_or("<not-set>"));

    // save any shell history items in the store
    for input in file_utils::read_stdin(1)? {
        actions::add_shell_action(&mut store, &input, epic)?;
    }
    Ok(())
}