use backends::schema::commands;
use db;
use diesel;
use diesel::prelude::*;
use lib_error::*;
use Connection;

/// Fetch the id for the given epic, if present.
pub fn fetch_id(connection: &Connection, kind: &str, command: &str) -> Result<Option<i32>> {
    let existing = commands::dsl::commands
        .filter(commands::dsl::command.eq(&command))
        .filter(commands::dsl::kind.eq(&kind))
        .select(commands::dsl::id)
        .load::<Option<i32>>(connection)?;
    Ok(existing.iter().next().map(|a| a.expect("must have id")))
}

/// Fetch or create an entry in the commands table matching the passed in name.
pub fn fetch_or_create_id(connection: &Connection, kind: &str, command: &str) -> Result<i32> {
    match fetch_id(connection, kind, command)? {
        Some(existing) => {
            let page_id = db::pages::fetch_id(connection, command)?;
            let find_clause = commands::dsl::commands.filter(commands::dsl::id.eq(existing));
            diesel::update(find_clause)
                .set(commands::dsl::page_id.eq(page_id))
                .execute(connection)?;
            Ok(existing)
        }
        None => {
            let page_id = db::pages::fetch_id(connection, command)?;
            diesel::insert_into(commands::table)
                .values((
                    commands::dsl::command.eq(command),
                    commands::dsl::kind.eq(kind),
                    commands::dsl::page_id.eq(page_id),
                )).execute(connection)?;
            match fetch_id(connection, kind, command) {
                Err(e) => Err(e),
                Ok(Some(id)) => Ok(id),
                Ok(None) => Err("did not get id after inserting location".into()),
            }
        }
    }
}

// Fetch all the commands
#[allow(dead_code)]
pub fn fetch_commands(connection: &Connection) -> Result<Vec<String>> {
    let entries = commands::dsl::commands
        .select(commands::dsl::command)
        .load::<String>(connection)?;
    Ok(entries)
}

// Re-link all the `commands` that are urls to their optional entry in `pages`.
pub fn link_pages(connection: &Connection) -> Result<()> {
    for (url, id, page_id) in commands::dsl::commands
        .select((
            commands::dsl::command,
            commands::dsl::id,
            commands::dsl::page_id,
        )).filter(commands::dsl::kind.eq(&"url"))
        .load::<(String, Option<i32>, Option<i32>)>(connection)?
    {
        println!("looking for page of {}", url);
        if let Some(connect_id) = db::pages::fetch_id(connection, &url)? {
            if page_id.is_none() || page_id.unwrap() != connect_id {
                let find_clause =
                    commands::dsl::commands.filter(commands::dsl::id.eq(id.expect("command id")));
                diesel::update(find_clause)
                    .set(commands::dsl::page_id.eq(connect_id))
                    .execute(connection)?;
            }
        }
    }
    Ok(())
}
