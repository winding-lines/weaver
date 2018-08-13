//! Entity used when logging a new action. Could be collapsed with FormattedAction.

use config::Environment;
use lib_error::Result;
use libc::getppid;
use sys_info;
use ::date::now;

/// Data structure to create a new action.
#[derive(Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAction {
    /// Time when the command was executed.
    pub executed: String,
    /// Shell or url.
    pub kind: String,
    /// The actual command.
    pub command: String,
    /// Path for shell, other information for URL.
    pub location: Option<String>,
    /// Epic, if known.
    pub epic: Option<String>,
    /// Host where this command was executed.
    pub host: String,
    /// For the shell process ID of the parent process which created this
    pub parent_id: Option<String>,
    /// For the shell exit code of the process.
    pub status_code: Option<String>,
}

impl NewAction {
    pub fn build_from_shell(
        command: &str,
        exit_code: Option<i32>,
        env: &Environment,
    ) -> Result<NewAction> {
        let host = sys_info::hostname()?;
        let location = env.rebase_on_home(env.cwd.clone())?;
        let location = Some(Environment::encode_path(&location));
        let executed = now();
        let ppid = format!("{}", unsafe { getppid() });

        Ok(NewAction {
            executed,
            kind: "shell".into(),
            command: command.into(),
            location,
            epic: env.epic().map(String::from),
            host,
            parent_id: Some(ppid),
            status_code: exit_code.map(|a| format!("{}", a)),
        })
    }

    pub fn build_from_url(url: &str, location: &str, epic: Option<&str>) -> Result<NewAction> {
        let host = sys_info::hostname()?;
        let executed = now();
        Ok(NewAction {
            executed,
            kind: "url".into(),
            command: url.into(),
            location: Some(location.into()),
            epic: epic.map(String::from),
            host,
            parent_id: None,
            status_code: None,
        })
    }

    /// Name of the collection to use in the encrypted repo.
    pub fn collection_name() -> &'static str {
        "action"
    }

    /// Version of this data structure.
    pub fn version() -> &'static str {
        "1.0"
    }
}
