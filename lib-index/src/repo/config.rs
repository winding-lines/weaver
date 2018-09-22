// Critical information about the current repo that is saved on disk.
// Obviously the password (or equivalent) should not be part of this struct.
use bincode;
use lib_error::*;
use lib_goo::config::file_utils::app_folder;
use rust_sodium::crypto::pwhash::{gen_salt, Salt};
use std::fs::{create_dir, read, write};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
    salt_raw: Vec<u8>,
}

impl Config {
    // Read the Repo configuration from the default location, if it's present.
    pub fn read() -> Result<Option<Config>> {
        let path = Config::config_path()?;
        if !path.exists() {
            return Ok(None);
        };

        // Config file exists, read its content and deserialize it.
        let content = read(&path)?;

        let config = bincode::deserialize::<Config>(&content[..])
            .map(Some)
            .map_err(|_| "read repo config")?;
        Ok(config)
    }

    // Read an existing config or build a new one if
    pub fn read_or_build() -> Result<Config> {
        match Config::read()? {
            Some(c) => Ok(c),
            None => {
                // Generate some new salt for this repo
                let salt = gen_salt();
                let mut salt_raw = Vec::new();
                salt_raw.extend_from_slice(&salt.0);

                // Create the config and save it to disk.
                let config = Config { salt_raw };
                config.write()?;

                Ok(config)
            }
        }
    }

    // Where to store the configuration.
    fn config_path() -> Result<PathBuf> {
        let mut path = Self::repo_folder()?;
        path.push("repo.def");
        Ok(path)
    }

    pub fn is_config(path: &Path) -> bool {
        if let Some(file_name) = path.file_name() {
            return file_name == "repo.def";
        }
        false
    }

    // Write the configuration of this Repo. Overwriting the hash would make this store
    // unaccessible so guard against that.
    fn write(&self) -> Result<()> {
        let path = Config::config_path()?;
        if path.exists() {
            match Self::read() {
                Ok(Some(existing)) => {
                    if existing.salt_raw != self.salt_raw {
                        return Err("cannot overwrite existing repo config".into());
                    }
                }
                Err(e) => return Err(e),
                _ => {}
            }
        }
        let bin =
            bincode::serialize(self).map_err(|_| "bincode serialization error for repo config")?;
        write(&path, bin)?;
        Ok(())
    }

    // The folder where we should find the repo.
    pub fn repo_folder() -> Result<PathBuf> {
        let mut base_folder = app_folder()?;
        base_folder.push("text-repo");
        if !base_folder.exists() {
            create_dir(&base_folder)?;
        }
        Ok(base_folder)
    }

    pub fn salt(&self) -> Result<Salt> {
        match Salt::from_slice(&self.salt_raw[..]) {
            Some(s) => Ok(s),
            None => Err("cannot construct password salt".into()),
        }
    }
}
