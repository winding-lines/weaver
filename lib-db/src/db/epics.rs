use crate::backends::schema::epics;
use diesel;
use diesel::prelude::*;
use lib_error::*;
use crate::Connection;

/// Fetch the id for the given epic, if present.
pub fn fetch_id(connection: &Connection, name: &str) -> Result<Option<i32>> {
    let existing = epics::dsl::epics
        .filter(epics::dsl::name.eq(&name))
        .select(epics::dsl::id)
        .load::<Option<i32>>(connection)?;
    Ok(existing.iter().next().map(|a| a.expect("must have id")))
}

/// Fetch or create an entry in the epics table matching the passed in name.
pub fn fetch_or_create_id(connection: &Connection, path: &str) -> Result<i32> {
    match fetch_id(connection, path)? {
        Some(existing) => Ok(existing),
        None => {
            diesel::insert_into(epics::table)
                .values(epics::dsl::name.eq(path))
                .execute(connection)?;
            match fetch_id(connection, path) {
                Err(e) => Err(e),
                Ok(Some(id)) => Ok(id),
                Ok(None) => Err(WeaverErrorKind::Generic("did not get id after inserting location").into()),
            }
        }
    }
}

/// Fetch all the epics
#[allow(dead_code)]
pub fn fetch_all(connection: &Connection) -> Result<Vec<String>> {
    let entries = epics::dsl::epics
        .select(epics::dsl::name)
        .load::<String>(connection)?;
        Ok(entries)
}
