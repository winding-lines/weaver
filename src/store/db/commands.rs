use ::store::backends::schema::commands;
use ::store::Connection;
use diesel;
use diesel::prelude::*;
use weaver_error::{Result, ResultExt};

/// Fetch the id for the given epic, if present.
pub fn fetch_id(connection: &Connection, kind: &str, command: &str) -> Result<Option<i32>> {
    let existing = commands::dsl::commands
        .filter(commands::dsl::command.eq(&command))
        .filter(commands::dsl::kind.eq(&kind))
        .select(commands::dsl::id)
        .load::<Option<i32>>(connection)
        .chain_err(|| "testing existence in commands table")?;
    Ok(existing.iter().next().map(|a| a.expect("must have id")))
}

/// Fetch or create an entry in the commands table matching the passed in name.
pub fn fetch_or_create_id(connection: &Connection, kind: &str, command: &str) -> Result<i32> {
    match fetch_id(connection, kind, command)? {
        Some(existing) => Ok(existing),
        None => {
            diesel::insert_into(commands::table)
                .values((commands::dsl::command.eq(command), commands::dsl::kind.eq(kind)))
                .execute(connection)
                .chain_err(|| "insert into commands")?;
            match fetch_id(connection, kind, command) {
                Err(e) => Err(e),
                Ok(Some(id)) => Ok(id),
                Ok(None) => Err("did not get id after inserting location".into())
            }
        }
    }
}

// Fetch all the commands
#[allow(dead_code)]
pub fn fetch_commands(connection: &Connection) -> Result<Vec<String>> {
    commands::dsl::commands
        .select(commands::dsl::command)
        .load::<String>(connection)
        .chain_err(|| "fetch all commands")
}
