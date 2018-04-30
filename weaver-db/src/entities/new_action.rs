use chrono::prelude::*;
use std::env;
use sys_info;
use weaver_error::{Result, ResultExt};

pub struct NewAction<'a> {
    pub executed: String,
    pub kind: &'a str,
    pub command: &'a str,
    pub location: Option<String>,
    pub epic: Option<&'a str>,
    pub host: String,
}

impl<'a> NewAction<'a> {

    pub fn build_from_shell(command: &'a str, epic: Option<&'a str>) -> Result<NewAction<'a>> {
        let host = sys_info::hostname()?;
        let cwd = env::current_dir()
            .chain_err(|| "save command")?;
        let location = Some(cwd.as_path().to_string_lossy().into());
        let executed = now();
        Ok(NewAction {
            executed: executed,
            kind: "shell",
            command: &command,
            location,
            epic,
            host: host,
        })
    }

    pub fn build_from_url( url: &'a str, location: &'a str, epic: Option<&'a str>) -> Result<NewAction<'a>> {
        let host = sys_info::hostname()?;
        let executed = now();
        Ok(NewAction {
            executed: executed,
            kind: "url",
            command: url,
            location: Some(location.into()),
            epic,
            host: host,
        })
    }


}


fn now() -> String {
    let utc: DateTime<Utc> = Utc::now();
    utc.to_rfc3339()
}

