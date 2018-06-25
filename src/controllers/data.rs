use ::cli::DataSubCommand;
use std::fs::read;
use std::path::PathBuf;
use std::process::Command;
use weaver_db::config::file_utils;
use weaver_db::RealStore;
use weaver_index::Repo;
use weaver_error::*;

/// Execute subcommands for the Data command.
pub fn run(_store: &RealStore, command: &DataSubCommand) -> Result<()> {
    match command {
        DataSubCommand::Sqlite => {
            sqlite()
        },
        DataSubCommand::Create => {
           RealStore::create_database()
        },
        DataSubCommand::Encrypt(filename) => {
            let repo = Repo::build()?;
            let path = PathBuf::from(filename);
            let content = read(&path)?;
            let handle = repo.add(&content)?;
            println!("{}", handle);
            Ok(())
        }
        DataSubCommand::Decrypt(handle) => {
            let repo = Repo::build()?;

            let decoded = repo.read(&handle)?;
            println!("{}", String::from_utf8(decoded).unwrap());
            Ok(())
        }
    }
}

fn sqlite() -> Result<()> {
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

