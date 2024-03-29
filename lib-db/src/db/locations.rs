use crate::backends::schema::locations;
use diesel;
use diesel::prelude::*;
use lib_error::*;
use crate::Connection;

/// Fetch the id for the given location, if present.
pub fn fetch_id(connection: &Connection, path: &str) -> Result<Option<i32>> {
    let existing = locations::dsl::locations
        .filter(locations::dsl::location.eq(&path))
        .select(locations::dsl::id)
        .load::<Option<i32>>(connection)?;
    Ok(existing.iter().next().map(|a| a.expect("must have isd")))
}

/// Fetch or create an entry in the locations table matching the passed in location.
pub fn fetch_or_create_id(connection: &Connection, path: &str) -> Result<i32> {
    match fetch_id(connection, path)? {
        Some(existing) => Ok(existing),
        None => {
            diesel::insert_into(locations::table)
                .values(locations::dsl::location.eq(path))
                .execute(connection)?;
            match fetch_id(connection, path) {
                Err(e) => Err(e),
                Ok(Some(id)) => Ok(id),
                Ok(None) => Err("did not get id after inserting location".into()),
            }
        }
    }
}

/// Fetch all locations.
#[allow(dead_code)]
pub fn fetch_all(connection: &Connection) -> Result<Vec<String>> {
    let entries = locations::dsl::locations
        .select(locations::dsl::location)
        .load::<String>(connection)?;
    Ok(entries)
}
