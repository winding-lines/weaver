use ::cli;
use std::fs;
use std::path::{Path, PathBuf};
use super::shell_proxy;
use walkdir::WalkDir;
use weaver_db::config::file_utils::{app_folder, read_content, write_content};
use weaver_db::entities::Flow;
use weaver_error::*;

/// Return the global or local folder for flows.
fn flows_folder(global: bool) -> Result<PathBuf> {
    let mut path = if global {
        app_folder()?
    } else {
        PathBuf::new()
    };
    path.push("flows");
    if !path.exists() {
        fs::create_dir(&path).chain_err(|| "create flows folder")?;
    }
    Ok(path)
}

/// Load the flows in the given folder.
pub fn load_folder(path: &Path, out: &mut Vec<Flow>) -> Result<()> {
    for entry in WalkDir::new(path) {
        let entry = entry.chain_err(|| "listing flows")?;
        let path = entry.path();
        if let Some(os_name) = path.file_name() {
            if let Some(name) = os_name.to_str() {
                if entry.file_type().is_file() && name.ends_with(".flow.json") {
                    let flow = read_content(path)
                        .and_then(|c| Flow::load_from_string(&c))
                        .chain_err(|| format!("loading flow from {:?}", path))?;
                    out.push(flow);
                }
            }
        }
    }
    Ok(())
}

/// Load all the flows both global and application.
pub fn load() -> Result<Vec<Flow>> {
    let mut out = Vec::new();
    load_folder(&flows_folder(true)?, &mut out)?;
    load_folder(&flows_folder(false)?, &mut out)?;
    Ok(out)
}


/// From all the Flows recommend the ones which are active for the current state,
/// i.e. their preconditions match.
pub fn active() -> Result<Vec<String>> {
    let mut out = Vec::new();
    for flow in load()? {
        if flow.matches() {
            out.push(flow.name.clone());
        }
    }
    Ok(out)
}

/// Run the actions in this particular flow.
fn run_flow(flow: &Flow) -> Result<()> {
    for action in &flow.actions {
        println!("  {}", action);
        shell_proxy::run(action).chain_err(|| "running flow action")?;
    }
    Ok(())
}

/// Run the actions in the Flow with the given name.
pub fn run<T>(name: T) -> Result<()>
    where T: AsRef<str> {
    for flow in &load()? {
        if flow.name.as_str().eq(name.as_ref()) {
            return run_flow(flow);
        }
    }
    Err(Error::from_kind(ErrorKind::from("flow not found")))
}

/// Create a flow with the given name, in the global location or not,
/// and with the given actions.
pub fn create(name: String, global: bool, actions: Vec<String>) -> Result<()> {
    let mut path = flows_folder(global)?;
    path.push(format!("{}.flow.json", name));
    let flow = Flow {
        name,
        preconditions: vec![],
        actions,
    };
    let data = flow.to_str()?;
    write_content(&path, &data).chain_err(|| "create flow")
}

/// Recommend a list of Flows for the current user.
pub fn recommend() -> Result<()> {
    let active = active()
        .chain_err(|| "getting active flows")?;

    let mut displayed = 0;
    for name in &active {
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
