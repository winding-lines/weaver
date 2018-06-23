use chrono::prelude::*;
use cli::ServerConfig;
use daemonize::Daemonize;
use std::fs;
use std::net::{TcpListener, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use weaver_db::config::{file_utils, ServerRun};
use weaver_db::RealStore;
use weaver_error::*;
use weaver_web;

fn server_folder() -> Result<PathBuf> {
    file_utils::app_folder().and_then(|mut path| {
        path.push("server");
        if !path.exists() {
            fs::create_dir(&path).chain_err(|| "create server folder")?;
        }
        Ok(path)
    })
}

fn pid_file() -> Result<PathBuf> {
    server_folder().map(|mut s| {
        s.push("server.pid");

        s
    })
}

fn is_listening(http_addr: &str) -> bool {
    let addr = match http_addr.to_socket_addrs().map(|ref mut i| i.next()) {
        Ok(Some(a)) => a,
        Ok(_) | Err(_) => panic!("unable to resolve listener address"),
    };

    match TcpListener::bind(addr) {
        Ok(listener) => {
            // We were able to bind to the address => no server is listening.
            drop(listener);
            false
        }
        Err(_) => {
            debug!("Error binding to {}, assume the server is running.", http_addr);
            true
        }
    }
}


pub struct Server;

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
pub fn start(run: &ServerRun, config: &ServerConfig, store: Arc<RealStore>) -> Result<Server> {
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
    let _actix = weaver_web::Server::start(&actix_address, store)?;

    Ok(Server)
}

pub fn is_running(config: &ServerConfig) -> bool {
    is_listening(&config.actix_address)
}

pub fn check(config: &ServerConfig) -> Result<()> {
    println!("actix listening {}", is_listening(&config.actix_address));
    Ok(())
}
