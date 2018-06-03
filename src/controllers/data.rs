use ::cli::DataSubCommand;
use std::process::Command;
use weaver_db::config::file_utils;
use weaver_db::RealStore;
use weaver_error::*;

/// Execute subcommands for the Data command.
pub fn run(_store: &RealStore, command: &DataSubCommand) -> Result<()> {
    match command {
        DataSubCommand::Sqlite => {
            sqlite()
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

