#![allow(proc_macro_derive_resolution_fallback)]
use crate::backends::schema::url_restrictions;
use crate::Connection;
use diesel;
use diesel::prelude::*;
use lib_error::{Result as WResult, WeaverError};
use std::str::FromStr;
use std::string::ToString;

// Url (patterns) to not log, by default all urls are logged.
pub const NO_LOG: &str = "!log";

// Url (patterns) to index, by default no urls get indexed so this is a white list.
pub const DO_INDEX: &str = "ndx";

// Url (patterns) to not index.
pub const NO_INDEX: &str = "!ndx";

// Url (patterns) to not display.
pub const HIDDEN: &str = "hide";

pub enum StorePolicy {
    NoLog,
    NoIndex,
    DoIndex,
    Hidden,
}

impl FromStr for StorePolicy {
    type Err = WeaverError;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        match s {
            NO_LOG => Ok(StorePolicy::NoLog),
            DO_INDEX => Ok(StorePolicy::DoIndex),
            NO_INDEX => Ok(StorePolicy::NoIndex),
            HIDDEN => Ok(StorePolicy::Hidden),
            _ => Err(WeaverError::from(format!("cannot parse url policy from {}", s))),
        }
    }
}

impl ToString for StorePolicy {
    fn to_string(&self) -> String {
        match *self {
            StorePolicy::NoLog => NO_LOG.into(),
            StorePolicy::NoIndex => NO_INDEX.into(),
            StorePolicy::DoIndex => DO_INDEX.into(),
            StorePolicy::Hidden => HIDDEN.into(),
        }
    }
}

// Fetch the id of the entry, if it already exists
fn fetch_id(connection: &Connection, ur: &UrlRestriction) -> WResult<Option<i32>> {
    let existing = url_restrictions::dsl::url_restrictions
        .filter(url_restrictions::dsl::url_expr.eq(&ur.url_expr))
        .filter(url_restrictions::dsl::kind.eq(&ur.kind))
        .filter(url_restrictions::dsl::title_match.eq(&ur.title_match))
        .filter(url_restrictions::dsl::body_match.eq(&ur.body_match))
        .select(url_restrictions::dsl::id)
        .load::<Option<i32>>(connection)?;
    Ok(existing.iter().next().map(|a| a.expect("must have id")))
}

// Insert a new restriction, unless it is already present.
pub fn insert(connection: &Connection, ur: UrlRestriction) -> WResult<()> {
    let existing = fetch_id(connection, &ur)?;
    if existing.is_none() {
        diesel::insert_into(url_restrictions::table)
            .values((
                url_restrictions::dsl::kind.eq(ur.kind),
                url_restrictions::dsl::url_expr.eq(ur.url_expr),
                url_restrictions::dsl::title_match.eq(ur.title_match),
                url_restrictions::dsl::body_match.eq(ur.body_match),
            )).execute(connection)?;
    }
    Ok(())
}

#[derive(Debug,::serde::Serialize, ::serde::Deserialize, Queryable, Default)]
#[allow(dead_code)]
pub struct UrlRestriction {
    pub id: Option<i32>,
    pub kind: String,
    pub url_expr: String,
    pub title_match: Option<String>,
    pub body_match: Option<String>,
}

impl UrlRestriction {
    pub fn with_url(kind: &StorePolicy, url: &str) -> UrlRestriction {
        UrlRestriction {
            kind: kind.to_string(),
            url_expr: url.into(),
            ..UrlRestriction::default()
        }
    }
}

pub fn fetch_all(connection: &Connection) -> WResult<Vec<UrlRestriction>> {
    let entries = url_restrictions::dsl::url_restrictions.load::<UrlRestriction>(connection)?;
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::SqlStoreInMemory;
    use crate::SqlProvider;

    #[test]
    fn test_only_url() {
        let connection = SqlStoreInMemory::build(|_| Ok(()))
            .connection()
            .expect("test connection");

        let gen = || UrlRestriction {
            kind: NO_INDEX.into(),
            url_expr: "a".into(),
            ..UrlRestriction::default()
        };

        insert(&connection, gen()).unwrap();
        let all = fetch_all(&connection).unwrap();
        assert_eq!(1, all.len());

        // same data should only be inserted once
        insert(&connection, gen()).unwrap();
        assert_eq!(1, all.len());
    }

    #[test]
    fn test_url_and_title() {
        let connection = SqlStoreInMemory::build(|_| Ok(()))
            .connection()
            .expect("test connection");

        let gen = || UrlRestriction {
            kind: NO_INDEX.into(),
            url_expr: "a".into(),
            title_match: Some("b".into()),
            ..UrlRestriction::default()
        };

        insert(&connection, gen()).unwrap();
        let all = fetch_all(&connection).unwrap();
        assert_eq!(1, all.len());

        // same data should only be inserted once
        insert(&connection, gen()).unwrap();
        assert_eq!(1, all.len());

        // fetching only url should not return anything
        let url = UrlRestriction {
            kind: NO_INDEX.into(),
            url_expr: "a".into(),
            ..UrlRestriction::default()
        };
        let fetched = fetch_id(&connection, &url).unwrap();
        assert!(fetched.is_none());
    }
}
