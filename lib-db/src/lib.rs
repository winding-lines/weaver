//! Database layer for [Weaver Project](../weaver_project/index.html).
//!
//!

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

pub use crate::db::actions2;
pub use crate::db::pages;
pub use crate::db::url_restrictions;
use diesel::sqlite::SqliteConnection;
use lib_error::*;
use lib_goo::config::file_utils;
use std::fs;
use std::path::PathBuf;

mod backends;
mod db;
pub mod setup;
pub mod store_policies;
pub mod test_helpers;
pub mod topics;

pub type Connection = SqliteConnection;

pub struct SqlStore {}

embed_migrations!("../migrations");

pub trait SqlProvider {
    fn connection(&self) -> Result<Connection>;
}

impl SqlStore {
    pub fn build() -> Result<SqlStore> {
        Ok(SqlStore {})
    }

    pub fn create_database_if_missing() -> Result<()> {
        let path = file_utils::default_database()?;
        if path.exists() {
            return Ok(());
        };
        Self::create_database()
    }

    pub fn backup_database() -> Result<PathBuf> {
        let path = file_utils::default_database()?;
        let backup = file_utils::backup_for_file(file_utils::DEFAULT_DB_NAME)?;
        fs::copy(path, backup.clone())?;

        Ok(backup)
    }

    pub fn create_or_backup_database() -> Result<()> {
        let path = file_utils::default_database()?;
        if path.exists() {
            Self::backup_database()?;

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
        let connection = SqliteConnection::establish(path_s.unwrap())?;
        embedded_migrations::run(&connection)
            .context("migration error".into())?;
        Ok(())
    }

    // Display information about the store, returns any errors.
    pub fn check() -> Result<()> {
        let path = file_utils::default_database()?;
        if !path.exists() {
            return Err("database file does not exists".into());
        }
        println!("SqlStore ok.");
        Ok(())
    }
}

impl SqlProvider for SqlStore {
    fn connection(&self) -> Result<Connection> {
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
        ::log::debug!("opening database {} ", &db_url);
        let connection = SqliteConnection::establish(&db_url)?;
        Ok(connection)
    }
}

// Recreate various connections between tables.
pub fn link_tables(connection: &Connection) -> Result<()> {
    db::commands::link_pages(connection)
}
