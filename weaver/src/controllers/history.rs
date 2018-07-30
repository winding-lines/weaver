use super::shell_proxy;
use clipboard::{ClipboardContext, ClipboardProvider};
use display;
use lib_error::*;
use lib_goo::config::{net, Destination, Environment, OutputKind};
use lib_rpc::client as rpc_client;
use std::path::Path;
use std::sync::Arc;

pub fn run(
    destination: &Destination,
    output_kind: &OutputKind,
    env: &Arc<Environment>,
) -> Result<()> {
    use lib_goo::config::Channel::*;

    let net::PaginatedActions {
        entries: mut actions,
        total: _total,
    } = rpc_client::recommendations(
        &destination,
        &net::RecommendationQuery {
            start: Some(0),
            length: Some(500),
            term: None,
        },
    )?;
    // rebase the command folders on the current work dir. This simplifies the UI interpretation.
    for mut a in actions.iter_mut() {
        if let Some(mut l) = a.location.as_mut() {
            let rebased = env.rebase(Path::new(&l).into())?;
            *l = Environment::encode_path(&rebased);
        }
    }
    // Put the most relevant and recent entries at the bottom.
    actions.reverse();

    // Run the main UI loop.
    let user_selection =
        display::main_screen::display(actions, &output_kind, Arc::clone(&env), destination)?;

    // Dispatch based on the selection.
    if let Some(action) = user_selection.action {
        match user_selection.kind {
            Some(OutputKind {
                channel: Run,
            }) => {
                if action.kind == "shell" {
                    shell_proxy::run(action.into_shell_command()).map(|_| ())
                } else {
                    shell_proxy::run(format!("open {}", action.name)).map(|_| ())
                }
            }
            Some(OutputKind {
                channel: Copy,
            }) => {
                eprintln!("Copying to clipboard: {}", action.name);
                if let Ok(mut ctx) = ClipboardContext::new() {
                    ctx.set_contents(action.into_shell_command())
                        .expect("set clipboard");
                }
                Ok(())
            }
            Some(OutputKind {
                channel: Print,
            }) => {
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
