use bincode;
use cli::{parse, ConfigAndCommand, DataSubCommand};
use lib_db::{self, pages, setup, topics, SqlProvider, SqlStore};
use lib_error::*;
use lib_goo::config::db::PasswordSource;
use lib_goo::config::file_utils;
use lib_goo::entities::PageContent;
use lib_index::repo::Repo;
use lib_index::{self, repo, Indexer, TantivyIndexer};
use std::fs::read;
use std::path::PathBuf;
use std::process::Command;

/// Main dispatch function;
pub fn run() -> Result<()> {
    use self::DataSubCommand::*;

    let ConfigAndCommand {
        password_source,
        command,
    } = parse();
    debug!("Executing cli command {:?}", command);
    let password_source = password_source.unwrap_or(PasswordSource::Keyring);

    // Initialize the crypto environment.
    lib_index::init()?;

    match command {
        Backup => {
            let name = SqlStore::backup_database()?;
            println!("Backup: {}", name.to_str().unwrap());

            Ok(())
        }
        Check => {
            let mut failures = 0;
            if let Err(e) = SqlStore::check() {
                println!("Failure in the sqlite store: {:?}", e);
                failures += 1;
            }
            if let Err(e) = repo::EncryptedRepo::check(&password_source) {
                println!("Failure in the text repo: {:?}", e);
                failures += 1;
            }
            if let Err(e) = TantivyIndexer::check() {
                println!("Failure in the indexer {:?}", e);
                failures += 1;
            }
            if let Err(e) = topics::TopicStore::check() {
                println!("Failure in the topic store {:?}", e);
                failures += 1;
            }
            if failures > 0 {
                Err(format!("{} stores failed", failures).into())
            } else {
                Ok(())
            }
        }
        Create => {
            SqlStore::create_or_backup_database()?;
            repo::EncryptedRepo::setup_if_needed(&password_source)?;
            let store = SqlStore::build()?;
            setup::populate_data(&store.connection()?)?;
            TantivyIndexer::setup_if_needed()?;
            Ok(())
        }
        Decrypt(collection, handle) => {
            let repo = repo::EncryptedRepo::build(&password_source)?;

            let decoded = repo.read(&collection, &handle)?;
            println!("{}", String::from_utf8(decoded).unwrap());
            Ok(())
        }
        DumpUrlPolicies => {
            let store = SqlStore::build()?;
            let policies = lib_db::store_policies::Restrictions::fetch(&store.connection()?)?;
            println!("\nLog all url accesses, with the following exceptions:");
            for p in policies.do_not_log {
                println!("  {:?}", p);
            }
            println!("\nFull text index only the following urls:");
            for p in policies.do_index {
                println!("  {:?}", p);
            }
            println!("\nException from the full text index list:");
            for p in policies.do_not_index {
                println!("  {:?}", p);
            }
            Ok(())
        }
        Encrypt(collection, filename) => {
            let repo = repo::EncryptedRepo::build(&password_source)?;

            // open source file
            let path = PathBuf::from(filename);
            let content = read(&path)?;

            // encrypt
            let handle = repo.add(&collection, &content)?;

            // print the handle
            println!("{}", handle);
            Ok(())
        }
        Noop => Ok(()),
        LinkCommandPages => {
            let store = SqlStore::build()?;
            let connection = store.connection()?;
            println!("Linking the commands and pages tables...");
            lib_db::link_tables(&connection)?;
            
            Ok(())
        },
        RebuildIndex => {

            lib_index::init()?;
            let repo = repo::EncryptedRepo::build(&password_source)?;
            let store = SqlStore::build()?;
            let connection = store.connection()?;

            TantivyIndexer::delete_all()?;
            TantivyIndexer::setup_if_needed()?;
            let indexer = TantivyIndexer::build()?;

            for entry in repo.list(&repo::Collection(PageContent::collection_name().into()))? {
                let decrypted = entry?;
                let page_content = bincode::deserialize::<PageContent>(decrypted.as_slice())
                    .chain_err(|| "cannot bindecode")?;

                // add to the indexer
                let handle = indexer.add(&page_content)?;

                // update/create the entry in pages
                let _page_id = pages::fetch_or_create_id(
                    &connection,
                    &page_content.url,
                    Some(&page_content.title),
                )?;
                println!("Indexed {} as {}", &page_content.url, handle);

            }

            println!("Linking the commands and pages tables...");
            lib_db::link_tables(&connection)?;

            Ok(())
        }
        Sqlite => execute_sqlite(),
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
