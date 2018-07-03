//! Database layer for [Weaver](../weaver/index.html).

extern crate chrono;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate log;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate lib_api;
extern crate weaver_error;


use ::lib_api::config::file_utils;
use diesel::sqlite::SqliteConnection;
use weaver_error::*;

mod db;
mod backends;
pub mod setup;

pub type Connection = SqliteConnection;
pub use db::actions2;
pub use db::epics;
pub use db::url_policies;


pub struct RealStore {
}

embed_migrations!("../migrations");

impl RealStore {
    pub fn new() -> Result<RealStore> {
        Ok(RealStore {
        })
    }

    pub fn create_database_if_missing() -> Result<()> {
        let path = file_utils::default_database()?;
        if path.exists() {
            return Ok(());
        };
        Self::create_database()
    }

    pub fn create_database() -> Result<()> {
        use diesel::Connection as DieselConnection;
        let path = file_utils::default_database()?;
        if path.exists() {
            return Err("output file already exists".into());
        }
        let path_s = path.to_str();
        if path_s.is_none() {
            return Err("bad path".into());
        }
        let connection = SqliteConnection::establish(path_s.unwrap())
            .chain_err(|| "opening up the connection")?;
        embedded_migrations::run(&connection).chain_err(|| "running migration")
    }

    pub fn connection(&self) -> Result<Connection> {
        use diesel::Connection as DieselConnection;
        let path = file_utils::default_database()?;
        if !path.exists() {
            return Err("database file does not exists".into());
        }
        let db_url = if let Some(value) = path.to_str() {
            String::from(value)
        } else {
            return Err("no database url".into());
        };
        debug!("opening database {} ", &db_url);
        let connection = SqliteConnection::establish(&db_url)
            .chain_err(|| format!("Cannot open database {}", db_url))?;
        Ok(connection)
    }

    // Display information about the store, returns any errors.
    pub fn check() -> Result<()> {
        let path = file_utils::default_database()?;
        if !path.exists() {
            return Err("database file does not exists".into());
        }
        println!("Store ok.");
        Ok(())

    }

}




