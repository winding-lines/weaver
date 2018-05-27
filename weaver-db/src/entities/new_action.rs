use chrono::prelude::*;
use sys_info;
use weaver_error::Result;
use ::config::Environment;

pub struct NewAction<'a> {
    pub executed: String,
    pub kind: &'a str,
    pub command: &'a str,
    pub location: Option<&'a str>,
    pub epic: Option<&'a str>,
    pub host: String,
}

impl<'a> NewAction<'a> {

    pub fn build_from_shell(command: &'a str, env: &'a Environment) -> Result<NewAction<'a>> {
        let host = sys_info::hostname()?;
        let location = Some(env.cwd());
        let executed = now();

        Ok(NewAction {
            executed,
            kind: "shell",
            command: &command,
            location,
            epic: env.epic(),
            host,
        })
    }

    pub fn build_from_url( url: &'a str, location: &'a str, epic: Option<&'a str>) -> Result<NewAction<'a>> {
        let host = sys_info::hostname()?;
        let executed = now();
        Ok(NewAction {
            executed,
            kind: "url",
            command: url,
            location: Some(location),
            epic,
            host,
        })
    }


}


fn now() -> String {
    let utc: DateTime<Utc> = Utc::now();
    utc.to_rfc3339()
}

