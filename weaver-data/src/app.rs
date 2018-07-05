use ::cli::{ConfigAndCommand, DataSubCommand, parse};
use bincode;
use lib_api::config::db::PasswordSource;
use lib_api::config::file_utils;
use lib_api::entities::PageContent;
use lib_db::{RealStore, setup};
use lib_error::*;
use lib_index::{self, Indexer, Repo};
use std::fs::read;
use std::path::PathBuf;
use std::process::Command;


/// Main dispatch function;
pub fn run() -> Result<()> {
    use self::DataSubCommand::*;

    let ConfigAndCommand { password_source, command } = parse();
    debug!("Executing cli command {:?}", command);
    let password_source = password_source.unwrap_or(PasswordSource::Keyring);

    // Initialize the crypto environment.
    lib_index::init()?;

    match command {
        Check => {
            let mut failures = 0;
            if let Err(e) = RealStore::check() {
                println!("Failure in the sqlite store: {:?}", e);
                failures += 1;
            }
            if let Err(e) = Repo::check(&password_source) {
                println!("Failure in the text repo: {:?}", e);
                failures += 1;
            }
            if let Err(e) = Indexer::check() {
                println!("Failure in the indexer {:?}", e);
                failures += 1;
            }
            if failures > 0 {
                Err(format!("{} stores failed", failures).into())
            } else {
                Ok(())
            }
        }
        Create => {
            RealStore::create_database_if_missing()?;
            Repo::setup_if_needed(&password_source)?;
            let store = RealStore::new()?;
            setup::populate_data(&store.connection()?)?;
            Indexer::setup_if_needed()?;
            Ok(())
        }
        Encrypt(filename) => {
            let repo = Repo::build(&password_source)?;

            // open source file
            let path = PathBuf::from(filename);
            let content = read(&path)?;

            // encrypt
            let handle = repo.add(&content)?;

            // print the handle
            println!("{}", handle);
            Ok(())
        }
        Decrypt(handle) => {
            let repo = Repo::build(&password_source)?;

            let decoded = repo.read(&handle)?;
            println!("{}", String::from_utf8(decoded).unwrap());
            Ok(())
        }
        Noop => {
            Ok(())
        }
        RebuildIndex => {
            lib_index::init()?;
            let repo = Repo::build(&password_source)?;
            Indexer::delete_all()?;
            Indexer::setup_if_needed()?;
            let indexer = Indexer::build()?;
            for entry in repo.list()? {
                let decrypted = entry?;
                let page_content = bincode::deserialize::<PageContent>(decrypted.as_slice())
                    .chain_err(|| "cannot bindecode")?;
                let handle = indexer.add(&page_content)?;
                println!("Indexed {} as {}", &page_content.url, handle);

            }
            Ok(())
        }
        Sqlite => {
            execute_sqlite()
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

