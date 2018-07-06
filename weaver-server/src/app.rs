//! Main function of the weaver server.
//!
use ::cli::{CommandAndConfig, parse, ServerSubCommand};
use chrono::prelude::*;
use cli::ServerRun;
use daemonize::Daemonize;
use lib_goo::config::{file_utils, ServerConfig};
use lib_goo::config::db::PasswordSource;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use lib_db::RealStore;
use lib_error::*;
use lib_index;
use lib_server;


pub fn run() -> Result<()> {
    use self::ServerSubCommand::*;

    let CommandAndConfig { command, server_config, password_source } = parse();
    debug!("Executing cli command {:?}", command);
    match command {
        Noop => {
            Ok(())
        }
        ServerSubCommand::Start(ref mode) => {
            start(mode, &server_config, &password_source).map(|_| ())
        }
        ServerSubCommand::Check => {
            server_config.check()
        }
    }
}

// The base folder of the server.
fn server_folder() -> Result<PathBuf> {
    file_utils::app_folder().and_then(|mut path| {
        path.push("server");
        if !path.exists() {
            fs::create_dir(&path).chain_err(|| "create server folder")?;
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
        fs::rename(out, out_renamed).chain_err(|| "file rename")
    } else {
        Ok(())
    }
}


/// Start a server and use a `Router` to dispatch requests
fn start(run: &ServerRun, config: &ServerConfig, password_source: &PasswordSource) -> Result<Server> {
    // Initialize the stores before any (optional) forking.
    let store = Arc::new(RealStore::build()?);
    lib_index::init()?;
    let repo = Arc::new(lib_index::Repo::build(password_source)?);

    match run {
        ServerRun::Foreground => {}
        ServerRun::Daemonize => {
            let pid_file_ = pid_file()?;
            let server_folder_ = server_folder()?;
            let timestamp = Utc::now().format("%Y-%m-%d_%H_%M_%S").to_string();
            rename_files(&server_folder_, "out", &timestamp)?;
            rename_files(&server_folder_, "err", &timestamp)?;
            let daemonize = Daemonize::new()
                .pid_file(pid_file_) // Every method except `new` and `start`
                .chown_pid_file(true)      // is optional, see `Daemonize` documentation
                .working_directory(&server_folder_) // for default behaviour.
                .redirect_dir(Some(server_folder_))
                .umask(0o022);    // Set umask, `0o027` by default.
            daemonize.start()
                .chain_err(|| "start in daemon mode")?;
            println!("Started in daemon mode");
        }
    }
    let actix_address = config.actix_address.clone();
    let _actix = lib_server::Server::start(&actix_address, store, repo)?;

    Ok(Server)
}


