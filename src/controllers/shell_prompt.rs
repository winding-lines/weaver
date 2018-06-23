use local_api;
use std::env;
use std::io::{self, Write};
use weaver_db::config::{Environment, file_utils};
use weaver_db::entities::NewAction;
use weaver_db::RealStore;
use weaver_error::*;

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

/// Internal function to process shell prompt
fn _run(store: &RealStore, env: &Environment) -> Result<()> {


    // output the current epic so that it can end up in the prompt
    print!("{}", env.epic().unwrap_or("<not-set>"));

    // save any shell history items in the store
    for input in file_utils::read_stdin(1)? {
        let action = NewAction::build_from_shell(&input, env)?;
        local_api::insert_action(&action, &store.destination())?;
    }
    Ok(())
}

/// Processes the actions when called from the PS1 prompt.
/// Translate errors to less verbose output.
pub fn run(store: &RealStore, env: &Environment) -> Result<()> {
    if _run(store, env).is_err() {
        print!(" err");
    };
    let _ = io::stdout().flush();
    Ok(())
}
