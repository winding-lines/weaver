//! Main function of the weaver server.
//!
use chrono::prelude::*;
use crate::cli::ServerRun;
use crate::cli::{parse, CommandAndConfig, ServerSubCommand};
use daemonize::{self, Daemonize};
use lib_db::SqlStore;
use lib_error::*;
use lib_goo::config::db::PasswordSource;
use lib_goo::config::{file_utils, ServerConfig};
use lib_index;
use lib_server;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{thread, time};

pub fn run() -> Result<()> {
    use self::ServerSubCommand::*;

    let CommandAndConfig {
        command,
        server_config,
        password_source,
    } = parse();
    debug!("Executing cli command {:?}", command);
    match command {
        Noop => Ok(()),
        ServerSubCommand::Start(ref mode) => {
            start(mode, &server_config, &password_source).map(|_| ())
        }
        ServerSubCommand::Check => server_config.check(),
    }
}

// The base folder of the server.
fn server_folder() -> Result<PathBuf> {
    file_utils::app_folder().and_then(|mut path| {
        path.push("server");
        if !path.exists() {
            fs::create_dir(&path)?;
        }
        Ok(path)
    })
}

// Where to serve the server's pid, when running in background mode.
fn pid_file() -> Result<PathBuf> {
    server_folder().map(|mut s| {
        s.push("server.pid");

        s
    })
}

// Placeholder struct for future expansion.
pub struct Server;

// Rename the log files to keep multiple versions around.
fn rename_files(server_folder: &Path, extension: &str, timestamp: &str) -> Result<()> {
    let out = PathBuf::from(server_folder);
    out.join(format!("server.{}", extension));
    if out.exists() && out.metadata()?.len() > 0 {
        let out_renamed = PathBuf::from(server_folder);
        out_renamed.join(format!("server-{}.{}", timestamp, extension));
        fs::rename(out, out_renamed)?;
    }
    Ok(())
}

/// Start a server and use a `Router` to dispatch requests
fn start(
    run: &ServerRun,
    config: &ServerConfig,
    password_source: &PasswordSource,
) -> Result<Server> {
    // Initialize the stores before any (optional) forking.
    let store = Arc::new(SqlStore::build()?);
    lib_index::init()?;
    let repo = Arc::new(lib_index::repo::EncryptedRepo::build(password_source)?);

    match run {
        ServerRun::Foreground => {}
        ServerRun::Daemonize(retries) => {
            let pid_file_ = pid_file()?;
            let server_folder_ = server_folder()?;
            let timestamp = Utc::now().format("%Y-%m-%d_%H_%M_%S").to_string();
            rename_files(&server_folder_, "out", &timestamp)?;
            rename_files(&server_folder_, "err", &timestamp)?;
            let retries = retries.unwrap_or(1);
            let mut last_status = Err(daemonize::DaemonizeError::Fork);
            for _i in 0..retries {
                let daemonize = Daemonize::new()
                    .pid_file(pid_file_.clone())
                    .chown_pid_file(true)
                    .working_directory(&server_folder_.clone())
                    .redirect_dir(Some(server_folder_.clone()))
                    .umask(0o022);

                last_status = daemonize.start();
                if last_status.is_ok() {
                    break;
                }
                let retry_sleep = 5;
                println!(
                    "... failed {:?}, retrying in {} seconds",
                    last_status, retry_sleep
                );
                thread::sleep(time::Duration::from_secs(retry_sleep));
            }
            if last_status.is_err() {
                return Err(format!("daemon start {:?}", last_status).into());
            }
            println!("Started in daemon mode");
        }
    }
    let _actix = lib_server::Server::start(config.http_port, config.https_port, store, repo)?;

    Ok(Server)
}
