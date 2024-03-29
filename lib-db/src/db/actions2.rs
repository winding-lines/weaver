#![allow(proc_macro_derive_resolution_fallback)]
use crate::backends::schema::*;
use crate::db;
use diesel;
use diesel::prelude::*;
use lib_error::*;
use lib_goo::config::net::Pagination;
use lib_goo::date;
use lib_goo::entities::{ActionId, FormattedAction, NewAction, RecommendReason};
use crate::Connection;

#[derive(Queryable, Debug)]
#[allow(dead_code)]
struct Action2 {
    id: Option<i32>,
    command_id: Option<i32>,
    executed: String,
    location_id: Option<i32>,
    epic_id: Option<i32>,
    sent: Option<bool>,
    annotation: Option<String>,
    host_id: Option<i32>,
    status: Option<i32>,
}

#[derive(Queryable, Debug)]
#[allow(dead_code)]
struct Location {
    id: Option<i32>,
    location: String,
}

#[allow(dead_code)]
#[derive(Queryable, Debug)]
struct Epic {
    id: Option<i32>,
    name: String,
}

#[allow(dead_code)]
#[derive(Queryable, Debug)]
struct Page {
    id: Option<i32>,
    normalized_url: String,
    title: Option<String>,
}

#[derive(Queryable, Debug)]
#[allow(dead_code)]
struct Command {
    id: Option<i32>,
    kind: String,
    command: String,
    page_id: Option<i32>,
}

/// Count the number of actions.
pub fn count(connection: &Connection) -> Result<usize> {
    let first: i64 = actions2::table.count().get_result(connection)?;
    Ok(first as usize)
}

#[allow(dead_code)]
type Backend = diesel::sqlite::Sqlite;

/// Fetch all actions as FormattedActions, use the pagination settings for the range. If present
pub fn fetch(
    connection: &Connection,
    search: Option<&str>,
    pagination: &Pagination,
) -> Result<Vec<FormattedAction>> {
    // setup the table joins, need to use into_boxed() to handle conditional code.
    let mut joined = actions2::table
        .inner_join(commands::table.left_join(pages::table))
        .left_join(locations::table)
        .into_boxed();

    // Apply an optional filter
    if let Some(txt) = search {
        let like_clause = format!("%{}%", txt);
        ::log::info!("like clause '{}'", like_clause);
        joined = joined.filter(
            commands::dsl::command
                .like(like_clause.clone())
                .or(pages::dsl::title.like(like_clause)),
        );
    };

    // info!("sql {:?}", diesel::debug_query::<Backend, _>(&joined));
    // Note: in sqlite3 you cannot pass offset without limit.
    let loaded = joined
        .limit(pagination.length.unwrap_or(-1))
        .offset(pagination.start.unwrap_or(0))
        .load::<(Action2, (Command, Option<Page>), Option<Location>)>(connection)?;
    let mut out = Vec::new();
    for (action2, (command, page_rec), location_rec) in loaded {
        let when = date::Date::parse(&action2.executed).ok();

        let (name, location) = if let Some(page) = page_rec {
            // If we have a matching page and it has a title then use it's title for the name
            (
                page.title.unwrap_or_else(|| command.command.clone()),
                Some(command.command),
            )
        } else {
            let clean = if command.kind == "url" {
                // For urls the location is not useful, just leave as empty.
                None
            } else {
                // Otherwise defer to the location of the command.
                location_rec.map(|l| l.location)
            };
            (command.command, clean)
        };
        let formatted = FormattedAction {
            annotation: action2.annotation,
            id: action2
                .id
                .map(|a| ActionId::new(a as usize))
                .unwrap_or_default(),
            epic: None,
            kind: command.kind,
            name,
            location,
            reason: RecommendReason::default(),
            when,
        };
        out.push(formatted);
    }

    Ok(out)
}

// Return the last access time for the given command
pub fn last_access(connection: &Connection, command: &str) -> Result<Option<String>> {
    let entries = actions2::dsl::actions2
        .inner_join(commands::dsl::commands)
        .select(actions2::dsl::executed)
        .filter(commands::dsl::command.eq(command))
        .order(actions2::dsl::id.desc())
        .limit(1)
        .load::<(String)>(connection)?;
    Ok(entries.first().cloned())
}

/// Insert a new action in the database.
pub fn insert(connection: &Connection, action: &NewAction) -> Result<u64> {
    use diesel::Connection as DieselConnection;

    connection.transaction::<u64, _, _>(|| {
        let location_id = if let Some(path) = action.location.as_ref() {
            Some(db::locations::fetch_or_create_id(connection, &path)?)
        } else {
            None
        };
        let epic_id = if let Some(epic) = action.epic.as_ref() {
            Some(db::epics::fetch_or_create_id(connection, &epic)?)
        } else {
            None
        };
        let command_id =
            db::commands::fetch_or_create_id(connection, &action.kind, &action.command)?;
        let host_id = db::hosts::fetch_or_create_id(connection, &action.host)?;
        let entry = (
            actions2::dsl::command_id.eq(command_id),
            actions2::dsl::executed.eq(&action.executed),
            actions2::dsl::location_id.eq(location_id),
            actions2::dsl::epic_id.eq(epic_id),
            actions2::dsl::sent.eq(false),
            actions2::dsl::annotation.eq(String::new()),
            actions2::dsl::host_id.eq(host_id),
        );
        let count = diesel::insert_into(actions2::table)
            .values(entry)
            .execute(connection)?;
        if count != 1 {
            return Err(WeaverError::from(format!("bad insert count {} during migration", count)));
        }
        Ok(1)
    })
}

pub fn set_annotation(connection: &Connection, id: u64, annotation: &str) -> Result<(u64)> {
    let find_clause = actions2::dsl::actions2.filter(actions2::dsl::id.eq(id as i32));
    let id = diesel::update(find_clause)
        .set(actions2::dsl::annotation.eq(annotation))
        .execute(connection)?;
    Ok(id as u64)
}

#[cfg(test)]
mod tests {
    use lib_goo::config::net::*;
    use lib_goo::date::now;
    use lib_goo::entities::NewAction;
    use crate::test_helpers::SqlStoreInMemory;
    use crate::SqlProvider;

    #[test]
    fn test_insert_and_fetch() {
        let connection = SqlStoreInMemory::build(|_| Ok(()))
            .connection()
            .expect("test connection");

        let res = super::insert(&connection, &NewAction::default());
        assert!(res.is_ok(), format!("insert failed {:?}", res));

        let all = super::fetch(&connection, None, &Pagination::default());
        assert!(res.is_ok(), format!("fetch_all failed {:?}", res));

        let actions = all.unwrap();
        assert_eq!(actions.len(), 1);
    }

    #[test]
    fn test_insert_and_count() {
        let connection = SqlStoreInMemory::build(|_| Ok(()))
            .connection()
            .expect("test connection");

        let res = super::insert(&connection, &NewAction::default());
        assert!(res.is_ok(), format!("insert failed {:?}", res));

        let count = super::count(&connection).unwrap();

        assert_eq!(count, 1);
    }

    #[test]
    fn test_set_annotation() {
        let connection = SqlStoreInMemory::build(|_| Ok(()))
            .connection()
            .expect("test connection");

        let res = super::insert(&connection, &NewAction::default());
        assert!(res.is_ok(), format!("insert failed {:?}", res));

        let update = super::set_annotation(&connection, 1, "ha-not-ate");
        assert!(update.is_ok(), format!("update failed {:?}", update));

        let all = super::fetch(&connection, None, &Pagination::default());
        assert!(res.is_ok(), format!("fetch_all failed {:?}", res));

        assert_eq!(
            all.unwrap().get(0).unwrap().annotation,
            Some(String::from("ha-not-ate"))
        );
    }

    fn new_action(name: &str) -> NewAction {
        NewAction {
            command: name.into(),
            ..NewAction::default()
        }
    }

    #[test]
    fn test_search() {
        let connection = SqlStoreInMemory::build(|_| Ok(()))
            .connection()
            .expect("test connection");
        for i in vec!["foo", "bar", "baz"] {
            super::insert(&connection, &new_action(i)).expect("insert");
        }
        let all = super::fetch(&connection, Some("ba"), &Pagination::default()).expect("fetch");
        assert_eq!(all.len(), 2);
        assert_eq!(
            all.iter().map(|a| a.name.as_str()).collect::<Vec<&str>>(),
            vec!["bar", "baz"]
        );
    }

    #[test]
    fn test_last_access() {
        let connection = SqlStoreInMemory::build(|_| Ok(()))
            .connection()
            .expect("test connection");
        let time = now();
        let url = "http://foo/last_access";
        let action = NewAction {
            command: url.into(),
            executed: time.clone().into(),
            ..NewAction::default()
        };
        super::insert(&connection, &action).expect("insert");
        let fetched = super::last_access(&connection, url)
            .expect("last_access")
            .unwrap();
        assert_eq!(fetched, time);
    }
}
