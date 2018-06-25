// Critical information about the current repo that is saved on disk.
// Obviously the password (or equivalent) should not be part of this struct.
use bincode;
use std::fs::{read, write, create_dir};
use std::path::PathBuf;
use weaver_db::config::file_utils::app_folder;
use rust_sodium::crypto::pwhash::{gen_salt,Salt};
use weaver_error::*;

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

        bincode::deserialize::<Config>(&content[..])
            .map(Some)
            .chain_err(|| "read repo config")
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
                let config = Config {
                    salt_raw,
                };
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

    // Write the configuration of this Repo. Overwriting the hash would make this store
    // unaccessible so guard against that.
    fn write(&self) -> Result<()> {
        let path = Config::config_path()?;
        if path.exists() {
            match Self::read() {
                Ok(Some(existing)) => {
                    if existing.salt_raw != self.salt_raw {
                        return Err("cannot overwrite existing repo config".into())
                    }
                },
                Err(e) => return Err(e),
                _ => {},
            }
        }
        let bin = bincode::serialize(self)
            .chain_err(|| "bincode serialization error for repo config")?;
        write(&path, bin).chain_err(|| "write repo config")
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

