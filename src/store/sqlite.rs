use ::errors::*;
use chrono::prelude::*;
use diesel;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use std::env;
use super::models::NewAction;
use super::schema::actions;


pub struct Store {
    connection: SqliteConnection
}

impl Store {
    pub fn new() -> Result<Store> {
        let _ = dotenv().chain_err(|| "store config")?;
        let db_url = env::var("DATABASE_URL")
            .chain_err(|| "store config")?;
        let connection = SqliteConnection::establish(&db_url)
            .chain_err(|| "store config")?;
        Ok(Store { connection })
    }

    pub fn add_shell(&self, command: &str) -> Result<()> {
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


