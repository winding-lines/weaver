use lib_error::*;
use lib_goo::config::Destination;
use lib_goo::config::{file_utils, Environment};
use lib_goo::entities::NewAction;
use local_api;
use std::env;
use std::io::{self, Write};

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

/// Do not save this input line in the database
fn reject_input(input: &str) -> bool {
    input.len() > 9 && &input[0..10] == "weaver_cmd"
}

/// Internal function to process shell prompt
fn _run(destination: &Destination, env: &Environment) -> Result<()> {
    // output the current epic so that it can end up in the prompt
    print!("{}", env.epic().unwrap_or("<not-set>"));

    // save any shell history items in the store
    for input in file_utils::read_stdin(1)? {
        if !reject_input(&input) {
            let action = NewAction::build_from_shell(&input, env)?;
            local_api::insert_action(&action, destination)?;
        }
    }
    Ok(())
}

/// Processes the actions when called from the PS1 prompt.
/// Translate errors to less verbose output.
pub fn run(destination: &Destination, env: &Environment) -> Result<()> {
    if _run(destination, env).is_err() {
        print!(" err");
    };
    let _ = io::stdout().flush();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reject() {
        assert!(reject_input("weaver_cmd"));
    }
}
