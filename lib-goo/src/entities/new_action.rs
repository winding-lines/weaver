//! Entity used when logging a new action. Could be collapsed with FormattedAction.

use chrono::prelude::*;
use sys_info;
use lib_error::Result;
use ::config::Environment;

#[derive(Deserialize, Serialize)]
pub struct NewAction {
    pub executed: String,
    pub kind: String,
    pub command: String,
    pub location: Option<String>,
    pub epic: Option<String>,
    pub host: String,
}

impl NewAction {

    pub fn build_from_shell(command: &str, env: &Environment) -> Result<NewAction> {
        let host = sys_info::hostname()?;
        let location = env.rebase_on_home(env.cwd.clone())?;
        let location = Some(Environment::encode_path(&location));
        let executed = now();

        Ok(NewAction {
            executed,
            kind: "shell".into(),
            command: command.into(),
            location,
            epic: env.epic().map(String::from),
            host,
        })
    }

    pub fn build_from_url( url: &str, location: &str, epic: Option<&str>) -> Result<NewAction> {
        let host = sys_info::hostname()?;
        let executed = now();
        Ok(NewAction {
            executed,
            kind: "url".into(),
            command: url.into(),
            location: Some(location.into()),
            epic: epic.map(String::from),
            host,
        })
    }


}


fn now() -> String {
    let utc: DateTime<Utc> = Utc::now();
    utc.to_rfc3339()
}

