use ::backends::schema::*;
use ::Connection;
use ::db;
use ::entities::{FormattedAction, NewAction};
use diesel;
use diesel::prelude::*;
use weaver_error::*;


#[derive(Queryable)]
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
}

#[derive(Queryable)]
#[allow(dead_code)]
struct Location {
    id: Option<i32>,
    location: String,
}

#[derive(Queryable)]
#[allow(dead_code)]
struct Epic {
    id: Option<i32>,
    name: String,
}

#[derive(Queryable)]
#[allow(dead_code)]
struct Command {
    id: Option<i32>,
    kind: String,
    command: String,
}

/// Fetch all actions as FormattedActions.
pub fn fetch_all(connection: &Connection) -> Result<Vec<FormattedAction>> {
    let joined = actions2::table
        .inner_join(commands::table)
        .left_join(locations::table)
        .left_join(epics::table)
        .load::<(Action2, Command, Option<Location>, Option<Epic>)>(connection)
        .chain_err(|| "joined load of actions2")?;
    let mut out = Vec::new();
    for (action2, command, location, epic) in joined.into_iter() {
        let formatted = FormattedAction {
            annotation: action2.annotation,
            id: action2.id.unwrap_or(0) as usize,
            epic: epic.map(|e| e.name),
            kind: command.kind,
            name: command.command,
            location: location.map(|l| l.location),
        };
        out.push(formatted);
    }

    Ok(out)
}

pub fn last_url(connection: &Connection) -> Result<Option<(String, String)>> {
    let entries = actions2::dsl::actions2
        .inner_join(commands::dsl::commands)
        .left_join(locations::table)
        .filter(commands::dsl::kind.eq("url"))
        .order(actions2::dsl::id.desc())
        .limit(1)
        .load::<(Action2, Command, Option<Location>)>(connection)
        .chain_err(|| "loading last url")?;
    let first = entries.into_iter().next();
    Ok(first.map(|(_, command, location)| (command.command, location.map(|l| l.location).unwrap_or("".into()))))
}

/// Insert a new action in the database.
pub fn insert(connection: &Connection, action: NewAction) -> Result<u64> {
    use diesel::Connection as DieselConnection;

    connection.transaction::<u64, _, _>(|| {
        let location_id = if let Some(path) = action.location {
            Some(db::locations::fetch_or_create_id(connection, &path)?)
        } else {
            None
        };
        let epic_id = if let Some(epic) = action.epic {
            Some(db::epics::fetch_or_create_id(connection, &epic)?)
        } else {
            None
        };
        let command_id = db::commands::fetch_or_create_id(connection, &action.kind, &action.command)?;
        let host_id = db::hosts::fetch_or_create_id(connection, &action.host)?;
        let migrated = (
            actions2::dsl::command_id.eq(command_id),
            actions2::dsl::executed.eq(action.executed),
            actions2::dsl::location_id.eq(location_id),
            actions2::dsl::epic_id.eq(epic_id),
            actions2::dsl::sent.eq(false),
            actions2::dsl::annotation.eq(String::new()),
            actions2::dsl::host_id.eq(host_id)
        );
        let count = diesel::insert_into(actions2::table)
            .values(migrated)
            .execute(connection)
            .chain_err(|| "inserting action2")?;
        if count != 1 {
            return Err(format!("bad insert count {} during migration", count).into());
        }
        Ok(1)
    })
}

pub fn set_annotation(connection: &Connection, id: u64, annotation: &str) -> Result<(u64)> {
    let find_clause = actions2::dsl::actions2.filter(actions2::dsl::id.eq(id as i32));
    diesel::update(find_clause)
        .set(actions2::dsl::annotation.eq(annotation))
        .execute(connection)
        .chain_err(|| "error updating annotation field")
        .map(|_| (id))
}

#[cfg(test)]
mod tests {
    use diesel;
    use ::entities::NewAction;

    embed_migrations!("../migrations");


    fn connection_with_tables() -> diesel::sqlite::SqliteConnection {
        use diesel::Connection as DieselConnection;
        use diesel::sqlite::SqliteConnection;

        let connection = SqliteConnection::establish(":memory:")
            .expect("in memory database");
        embedded_migrations::run(&connection)
            .expect("create tables");
        connection
    }

    fn new_action<'a>() -> NewAction<'a> {
        NewAction {
            executed: String::new(),
            kind: "",
            command: "",
            location: None,
            epic: None,
            host: String::new(),
        }
    }

    #[test]
    fn test_insert_and_fetch() {

        let connection = connection_with_tables();

        let res = super::insert(&connection, new_action());
        assert!(res.is_ok(), format!("insert failed {:?}", res));

        let all = super::fetch_all(&connection);
        assert!(res.is_ok(), format!("fetch_all failed {:?}", res));

        let actions = all.unwrap();
        assert_eq!(actions.len(), 1);
    }

    #[test]
    fn test_set_annotation() {
        let connection = connection_with_tables();

        let res = super::insert(&connection, new_action());
        assert!(res.is_ok(), format!("insert failed {:?}", res));

        let update = super::set_annotation(&connection, 1, "ha-not-ate");
        assert!(update.is_ok(), format!("update failed {:?}", update));

        let all = super::fetch_all(&connection);
        assert!(res.is_ok(), format!("fetch_all failed {:?}", res));

        assert_eq!(all.unwrap().get(0).unwrap().annotation, Some(String::from("ha-not-ate")));
    }
}