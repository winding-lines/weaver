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
extern crate sys_info;
extern crate weaver_error;


use ::config::file_utils;
use ::entities::Weaver;
use diesel::sqlite::SqliteConnection;
use weaver_error::*;

pub mod entities;
mod filtered_vec;
mod db;
mod backends;
pub mod config;
pub mod setup;

pub type Connection = SqliteConnection;
pub use db::actions2;
pub use db::epics;
pub use db::url_policies;
pub use filtered_vec::FilteredVec;

pub enum ApiConfig {
    Local,
    Remote(String),
}

pub enum Destination {
    Local(Result<Connection>),
    Remote(String),
}

pub struct RealStore {
    json_store: backends::json_store::JsonStore,
    config: ApiConfig,
}

embed_migrations!("../migrations");

impl RealStore {
    pub fn new(config: ApiConfig) -> Result<RealStore> {
        let json_store = backends::json_store::JsonStore::init()?;
        Ok(RealStore {
            json_store,
            config,
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

    pub fn destination(&self) -> Destination {
        match &self.config {
            ApiConfig::Local => Destination::Local(self.connection()),
            ApiConfig::Remote(ref a) => Destination::Remote(a.clone()),
        }
    }

    /// Save this epic name in the local storage,
    /// does not change the current store.
    pub fn save_epic(name: String) -> Result<()> {
        let mut store = backends::json_store::JsonStore::init()?;
        store.content.active_epic = Some(name);
        store.save()
    }

    pub fn epic(&self) -> Result<Option<String>> {
        // let _ = self.json_store.fresh()?;
        Ok(self.json_store.content.active_epic.clone())
    }


    pub fn weaver(&self) -> &Weaver {
        &self.json_store.content
    }

    /// Return the active epic in a format that can be displayed, i.e. empty string for None.
    pub fn epic_display(&mut self) -> String {
        let _ = self.json_store.fresh();
        match self.json_store.content.active_epic {
            Some(ref s) => s.clone(),
            None => String::from(""),
        }
    }
}




