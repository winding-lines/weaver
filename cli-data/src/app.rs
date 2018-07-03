use ::cli::{DataSubCommand, parse};
use lib_api::config::file_utils;
use std::fs::read;
use std::path::PathBuf;
use std::process::Command;
use weaver_db::{RealStore, setup};
use weaver_error::*;
use weaver_index::{Indexer, Repo};


/// Main dispatch function;
pub fn run() -> Result<()> {
    use self::DataSubCommand::*;

    let command = parse();
    debug!("Executing cli command {:?}", command);
    match command {
        Noop => {
            Ok(())
        }
        Sqlite => {
            execute_sqlite()
        }
        Create => {
            RealStore::create_database_if_missing()?;
            Repo::setup_if_needed()?;
            let store = RealStore::new()?;
            setup::populate_data(&store.connection()?)?;
            Ok(())
        }
        Encrypt(filename) => {
            let repo = Repo::build()?;
            let path = PathBuf::from(filename);
            let content = read(&path)?;
            let handle = repo.add(&content)?;
            println!("{}", handle);
            Ok(())
        }
        Decrypt(handle) => {
            let repo = Repo::build()?;

            let decoded = repo.read(&handle)?;
            println!("{}", String::from_utf8(decoded).unwrap());
            Ok(())
        }
        Check => {
            RealStore::check()?;
            Repo::check()?;
            Indexer::check()?;
            Ok(())
        }
    }
}

fn execute_sqlite() -> Result<()> {
    if let Some(db_path) = file_utils::default_database()?.to_str() {
        let open_cmd = format!(".open {}", db_path);
        let mut cmd = Command::new("sqlite3");
        cmd.arg("-cmd");
        cmd.arg(open_cmd);
        cmd.status()
            .map(|_exit_code| ())
            .chain_err(|| "running sqlite3")
    } else {
        Err("Bad db path".into())
    }
}

