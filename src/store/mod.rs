use ::config::file_utils;
use ::entities::Weaver;
use ::errors::*;
use diesel::sqlite::SqliteConnection;


pub mod actions;
mod db;
mod backends;

pub type Connection = SqliteConnection;

#[derive(StateData)]
pub struct RealStore {
    json_store: backends::json_store::JsonStore,
    pub connection: Connection,
}

fn open_sqlite() -> Result<Connection> {
    use diesel::Connection as DieselConnection;
    let db_url = if let Some(value) = file_utils::default_database()?.to_str() {
        String::from(value)
    } else {
        return Err(Error::from_kind(ErrorKind::from("no database url")));
    };
    debug!("opening database {} ", &db_url);
    let connection = SqliteConnection::establish(&db_url)
        .chain_err(|| format!("Cannot open database {}", db_url))?;
    Ok(connection)
}

impl RealStore {
    pub fn new() -> Result<RealStore> {
        backends::json_store::JsonStore::init()
            .and_then(|json_store| {
                open_sqlite()
                    .and_then(|connection| Ok(RealStore {
                        json_store,
                        connection,
                    }))
            })
    }


    pub fn set_epic(&mut self, name: String) -> Result<()> {
        self.json_store.fresh()?;
        self.json_store.content.active_epic = Some(name);
        self.json_store.save()
    }

    pub fn epic(&mut self) -> Result<Option<String>> {
        let _ = self.json_store.fresh()?;
        Ok(self.json_store.content.active_epic.clone())
    }

    pub fn epic_names(&mut self) -> Result<Vec<String>> {
        db::epics::fetch_all(&self.connection)
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




