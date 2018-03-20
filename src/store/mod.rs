use diesel::sqlite::SqliteConnection;
use ::errors::*;
use ::entities::Weaver;

pub mod actions;
mod backends;

#[derive(StateData)]
pub struct Store {
    json_store: backends::json_store::JsonStore,
    sqlite: backends::sqlite::Sqlite,
}

impl Store {
    pub fn new() -> Result<Store> {
        backends::json_store::JsonStore::init()
            .and_then(|json_store| {
                backends::sqlite::Sqlite::new()
                    .and_then(|sqlite| Ok(Store {
                        json_store,
                        sqlite,
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

    pub fn add_shell_action(&self, command: &str, epic: Option<&str>) -> Result<()> {
        self.sqlite.add_shell_action(command, epic)
    }

    pub fn add_url_action(&self, url: &str, epic: Option<&str>) -> Result<u64> {
        self.sqlite.add_url_action(url, epic)
    }

    pub fn sqlite_connection<'a>(&'a mut self) -> &'a SqliteConnection {
        self.sqlite.connection()
    }
}
