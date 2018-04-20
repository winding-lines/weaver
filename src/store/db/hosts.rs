use diesel;
use diesel::prelude::*;
use ::errors::*;
use ::store::backends::schema::hosts;
use ::store::Connection;

// Fetch the id for the given host, if present.
pub fn fetch_id(connection: &Connection, host: &str) -> Result<Option<i32>> {
    let existing = hosts::dsl::hosts
        .filter(hosts::dsl::name.eq(&host))
        .select(hosts::dsl::id)
        .load::<Option<i32>>(connection)
        .chain_err(|| "testing existence in hosts table")?;
    Ok(existing.iter().next().map(|a| a.expect("must have isd")))
}

/// Fetch or create an entry in the hosts table matching the passed in location.
pub fn fetch_or_create_id(connection: &Connection, host: &str) -> Result<i32> {
    match fetch_id(connection, host)? {
        Some(existing) => Ok(existing),
        None => {
            diesel::insert_into(hosts::table)
                .values(hosts::dsl::name.eq(host))
                .execute(connection)
                .chain_err(|| "insert into hosts")?;
            match fetch_id(connection, host) {
                Err(e) => Err(e),
                Ok(Some(id)) => Ok(id),
                Ok(None) => Err("did not get id after inserting location".into())
            }
        }
    }
}

/// Fetch all hosts.
#[allow(dead_code)]
pub fn fetch_all(connection: &Connection) -> Result<Vec<String>> {
    hosts::dsl::hosts
        .select(hosts::dsl::name)
        .load::<String>(connection)
        .chain_err(|| "fetch all")
}
