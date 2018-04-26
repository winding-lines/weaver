use ::entities::FormattedAction;
use ::db;
use ::Connection;
use chrono::prelude::*;
use std::env;
use sys_info;
use weaver_error::{Result, ResultExt};


pub struct NewAction<'a> {
    pub executed: &'a str,
    pub kind: &'a str,
    pub command: &'a str,
    pub location: Option<&'a str>,
    pub epic: Option<&'a str>,
    pub host: &'a str,
}

fn now() -> String {
    let utc: DateTime<Utc> = Utc::now();
    utc.to_rfc3339()
}

pub fn history<T: AsRef<str>>(connection: &Connection, _epic: &Option<T>) -> Result<Vec<FormattedAction>> {
    db::actions2::fetch_all(connection)
}


// Return the last url action from the store.
pub fn last_url(connection: &Connection) -> Result<Option<(String, String)>> {
    db::actions2::last_url(connection)
}

pub fn add_shell_action(connection: &Connection, command: &str, epic: Option<&str>) -> Result<(u64)> {
    let host = sys_info::hostname()?;
    let cwd = env::current_dir()
        .chain_err(|| "save command")?;
    let location = cwd.as_path().to_str();
    let executed = now();
    let insert = NewAction {
        executed: &executed,
        kind: "shell",
        command: &command,
        location,
        epic,
        host: &host,
    };
    db::actions2::insert(connection, insert)
}

pub fn add_url_action(connection: &Connection, url: &str, location: &str, epic: Option<&str>) -> Result<u64> {
    let host = sys_info::hostname()?;
    let executed = now();
    let insert = NewAction {
        executed: &executed,
        kind: "url",
        command: url,
        location: Some(location),
        epic,
        host: &host,
    };
    db::actions2::insert(connection, insert)
}

/* Kept for reference
// Migrate the store to the latest version
pub fn migrate(store: &RealStore) -> Result<()> {

    // Extract locations from actions table
    let values = actions::dsl::actions
        .load::<(Action)>(&store.connection)
        .chain_err(|| "loading locations field from actions table")?;

    diesel::delete(actions2::table)
        .execute(&store.connection)
        .chain_err(|| "delete actions2 before migrate")?;

    // Insert values in location table if it doesn't exist
    for action in values.into_iter() {

        let location_id = if let Some(path) = action.location {
            Some(db::locations::fetch_or_create_id(&store.connection, &path)?)
        } else {
            None
        };
        let epic_id = if let Some(epic) = action.epic {
            Some(db::epics::fetch_or_create_id(&store.connection, &epic)?)
        } else {
            None
        };
        let command_id = db::commands::fetch_or_create_id(&store.connection, &action.kind, &action.command )?;
        let migrated = (
            actions2::dsl::command_id.eq(command_id),
            actions2::dsl::executed.eq(action.executed),
            actions2::dsl::location_id.eq(location_id),
            actions2::dsl::epic_id.eq(epic_id),
            actions2::dsl::sent.eq(action.sent),
            actions2::dsl::annotation.eq(action.annotation)
        );
        let count = diesel::insert_into(actions2::table)
            .values(migrated)
            .execute(&store.connection)
            .chain_err(|| "inserting action2")?;
        if count != 1 {
            return Err(format!("bad insert count {} during migration", count).into());
        }
    }


    Ok(())
}
*/
