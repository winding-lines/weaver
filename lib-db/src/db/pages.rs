use crate::backends::schema::pages;
use diesel;
use diesel::prelude::*;
use lib_error::*;
use lib_goo::normalize;
use crate::Connection;

/// Fetch the id for the given page, if present.
pub fn fetch_id(connection: &Connection, url: &str) -> Result<Option<i32>> {
    let normalized_url = normalize::normalize_url(url)?;
    let existing = pages::dsl::pages
        .filter(pages::dsl::normalized_url.eq(&normalized_url))
        .select(pages::dsl::id)
        .load::<Option<i32>>(connection)?;
    Ok(existing.iter().next().map(|a| a.expect("must have id")))
}

/// Fetch or create an entry in the epics table matching the passed in name.
pub fn fetch_or_create_id(connection: &Connection, url: &str, title: Option<&str>) -> Result<i32> {
    let normalized_url = normalize::normalize_url(url)?;
    match fetch_id(connection, &normalized_url)? {
        Some(existing) => Ok(existing),
        None => {
            diesel::insert_into(pages::table)
                .values((
                    pages::dsl::normalized_url.eq(&normalized_url),
                    pages::dsl::title.eq(title),
                ))
                .execute(connection)?;
            match fetch_id(connection, &normalized_url) {
                Err(e) => Err(e),
                Ok(Some(id)) => Ok(id),
                Ok(None) => Err(WeaverErrorKind::Generic("did not get id after inserting location").into()),
            }
        }
    }
}
