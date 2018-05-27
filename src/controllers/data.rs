use ::cli::{DataSubCommand, TextIndexSubCommand};
use weaver_db::config::file_utils;
use weaver_error::*;
use std::process::Command;
use weaver_db::RealStore;
use weaver_index::Indexer;

/// Execute subcommands for the Data command.
pub fn run(_store: & RealStore, command: &DataSubCommand) -> Result<()> {
    match command {
        DataSubCommand::Sqlite => {
            sqlite()
        },
        DataSubCommand::TextIndex(TextIndexSubCommand::Create) => {
            index_create()
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

fn index_create() -> Result<()> {
    let indexer = Indexer::build()?;
    let _ = indexer.add("id1", "Title for the t e s t document", "this is just a test document")?;
    let found = indexer.search("test")?;
    println!("found {}", found.len());
    for one in found {
        println!(" {} -> {}", one.0, one.1);
    }
    Ok(())
}
