use super::weaver::Weaver;
use lib_error::*;
use lib_goo::config::file_utils;
use std::time::SystemTime;

#[derive(Default)]
pub(crate) struct JsonStore {
    pub content: Weaver,
    modified: Option<SystemTime>,
}

impl JsonStore {
    /// Initialize the weaver application/global configuration.
    pub fn init() -> Result<JsonStore> {
        let mut store = JsonStore::default();
        store.fresh()?;
        Ok(store)
    }

    /// Re-read the weaver configuration, if it has changed.
    pub fn fresh(&mut self) -> Result<()> {
        let mut path = file_utils::app_folder()?;
        path.push("weaver.json");

        // Check if the file exists, nothing to refresh if not.
        if !path.exists() {
            return Ok(());
        }

        // Check the last modified time, exit if no changes.
        let file_modified = Some(
            path.metadata()
                .context("metadata in fresh".into())?
                .modified()
                .context("modified check in fresh".into())?,
        );
        if file_modified == self.modified {
            return Ok(());
        }

        ::log::debug!("loading config {:?}", &path);

        let content = file_utils::read_content(&path)?;
        let content = Weaver::load_from_string(&content)?;
        self.content = content;
        self.modified = file_modified;

        Ok(())
    }

    /// Save the weaver application/global configuration.
    pub fn save(&mut self) -> Result<()> {
        let mut path = file_utils::app_folder()?;
        path.push("weaver.json");
        let content = self.content.to_str()?;
        file_utils::write_content(&path, &content)?;
        self.modified = Some(
            path.metadata()?
                .modified()
                .context("refresh date after save".into())?
        );
        Ok(())
    }
}
