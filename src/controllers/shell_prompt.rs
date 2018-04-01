use ::config::file_utils;
use ::errors::*;
use ::store::RealStore;
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
pub fn run(epic: Option<&str>) -> Result<()> {
    println!("{}", epic.unwrap_or("<not-set>"));
    for input in file_utils::read_stdin(1)? {
        let mut store = RealStore::new()?;
        store.add_shell_action(&input, epic)?;
    }
    Ok(())
}