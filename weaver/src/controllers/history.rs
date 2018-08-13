use super::shell_proxy;
use api::fetch_recommendations;
use clipboard::{ClipboardContext, ClipboardProvider};
use display;
use lib_error::*;
use lib_goo::config::{Destination, Environment, OutputKind};
use std::sync::Arc;

pub fn run(
    destination: &Destination,
    output_kind: &OutputKind,
    env: &Arc<Environment>,
) -> Result<()> {
    use lib_goo::config::Channel::*;

    let actions = fetch_recommendations(None, &destination, env)?;

    // Run the main UI loop.
    let user_selection =
        display::main_screen::display(actions, &output_kind, Arc::clone(&env), destination)?;

    // Dispatch based on the selection.
    if let Some(action) = user_selection.action {
        match user_selection.kind {
            Some(OutputKind { channel: Run }) => {
                if action.kind == "shell" {
                    shell_proxy::run(action.into_shell_command()).map(|_| ())
                } else {
                    shell_proxy::run(format!("open {}", action.name)).map(|_| ())
                }
            }
            Some(OutputKind { channel: Copy }) => {
                eprintln!("Copying to clipboard: {}", action.name);
                if let Ok(mut ctx) = ClipboardContext::new() {
                    ctx.set_contents(action.into_shell_command())
                        .expect("set clipboard");
                }
                Ok(())
            }
            Some(OutputKind { channel: Print }) => {
                if action.kind == "shell" {
                    println!("{}", action.into_shell_command());
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
