use super::shell_proxy;
use clipboard::{ClipboardContext, ClipboardProvider};
use display;
use lib_error::*;
use lib_goo::config::{Destination, Environment, OutputKind};
use local_api;
use std::path::Path;
use std::sync::Arc;

pub fn run(
    destination: &Destination,
    output_kind: &OutputKind,
    env: &Arc<Environment>,
) -> Result<()> {
    use lib_goo::config::Channel::*;

    let mut actions = local_api::history(&env, &destination)?;
    // rebase for easier UI interpretation.
    for mut a in actions.iter_mut() {
        if let Some(mut l) = a.location.as_mut() {
            let rebased = env.rebase(Path::new(&l).into())?;
            *l = Environment::encode_path(&rebased);
        }
    }
    let user_selection =
        display::main_screen::display(actions, &output_kind, Arc::clone(&env), destination)?;
    if let Some(action) = user_selection.action {
        match user_selection.kind {
            Some(OutputKind {
                channel: Run,
                ref content,
            }) => {
                if action.kind == "shell" {
                    shell_proxy::run(action.into_shell_command(content, &env)).map(|_| ())
                } else {
                    shell_proxy::run(format!("open {}", action.name)).map(|_| ())
                }
            }
            Some(OutputKind {
                channel: Copy,
                ref content,
            }) => {
                eprintln!("Copying to clipboard: {}", action.name);
                if let Ok(mut ctx) = ClipboardContext::new() {
                    ctx.set_contents(action.into_shell_command(content, &env))
                        .expect("set clipboard");
                }
                Ok(())
            }
            Some(OutputKind {
                channel: Print,
                ref content,
            }) => {
                if action.kind == "shell" {
                    println!("{}", action.into_shell_command(content, &env));
                } else {
                    println!("open {}", action.name);
                }
                Ok(())
            }
            None => {
                eprintln!("No action kind passed in");
                Ok(())
            }
        }
    } else {
        eprintln!("No command selected from history");
        Ok(())
    }
}
