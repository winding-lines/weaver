use ::config::file_utils;
use ::errors::*;
use chrono::prelude::*;
use diesel;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use std::env;
use super::models::NewAction;
use super::schema::actions;


pub struct Store {
    connection: SqliteConnection
}

impl Store {
    pub fn new() -> Result<Store> {
        let db_url = env::var("DATABASE_URL")
            .or_else(|_| {
                if let Some(value) = file_utils::default_database()?.to_str() {
                    Ok(String::from(value))
                } else {
                    return Err(Error::from_kind(ErrorKind::from("no database url")));
                }
            })?;
        debug!("opening database {} ", &db_url);
        let connection = SqliteConnection::establish(&db_url)
            .chain_err(|| format!("Cannot open database {}", db_url))?;
        Ok(Store { connection })
    }

    pub fn connection<'a>(&'a mut self) -> &'a SqliteConnection {
        &self.connection
    }

    pub fn add_shell_action(&self, command: &str, epic: Option<&str>) -> Result<()> {
        let cwd = env::current_dir()
            .chain_err(|| "save command")?;
        let location = cwd.as_path().to_str();
        let utc: DateTime<Utc> = Utc::now();
        let executed = utc.to_rfc3339();
        let insert = NewAction {
            executed: &executed,
            kind: "shell",
            command: &command,
            location,
            epic,
        };
        debug!("inserting {:?} in actions table", insert);
        let count = diesel::insert_into(actions::table)
            .values(&insert)
            .execute(&self.connection)
            .chain_err(|| "save command")?;
        if count == 1 {
            Ok(())
        } else {
            Err(Error::from_kind(ErrorKind::from("Got bad numbers of rows in insert")))
        }
    }
}


