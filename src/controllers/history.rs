use clipboard::{ClipboardContext, ClipboardProvider};
use display;
use local_api;
use std::sync::Arc;
use super::shell_proxy;
use weaver_db::config::{Environment, OutputKind};
use weaver_db::RealStore;
use weaver_error::*;


pub fn run(store: Arc<RealStore>, output_kind: &OutputKind, env: &Arc<Environment>) -> Result<()> {
    use weaver_db::config::Channel::*;

    let destination = store.destination();
    let actions = local_api::history(&env, &destination)?;
    let user_selection = display::main_screen(actions, &output_kind, Arc::clone(&env), store)?;
    if let Some(action) = user_selection.action {
        match user_selection.kind {
            Some(OutputKind { channel: Run, ref content }) => {
                if action.kind == "shell" {
                    shell_proxy::run(action.to_shell_command(content, &env))
                        .map(|_| ())
                } else {
                    shell_proxy::run(format!("open {}", action.name))
                        .map(|_| ())
                }
            }
            Some(OutputKind { channel: Copy, ref content }) => {
                eprintln!("Copying to clipboard: {}", action.name);
                if let Ok(mut ctx) = ClipboardContext::new() {
                    ctx.set_contents(action.to_shell_command(content, &env)).expect("set clipboard");
                }
                Ok(())
            }
            Some(OutputKind { channel: Print, ref content }) => {
                if action.kind == "shell" {
                    println!("{}", action.to_shell_command(content, &env));
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

