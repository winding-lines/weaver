use ::http_server;
use daemonize::Daemonize;
use std::fs;
use std::net::{TcpListener, ToSocketAddrs};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use weaver_db::config::{file_utils, ServerRun};
use weaver_db::RealStore;
use weaver_error::*;
use weaver_rpc;
use weaver_web;
use cli::ServerConfig;

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

/// Start a server and use a `Router` to dispatch requests
pub fn start(run: &ServerRun, config: &ServerConfig, store: Arc<RealStore>) -> Result<Server> {
    match run {
        ServerRun::Foreground => {}
        ServerRun::Daemonize => {
            let pid_file_ = pid_file()?;
            let server_folder_ = server_folder()?;
            let daemonize = Daemonize::new()
                .pid_file(pid_file_) // Every method except `new` and `start`
                .chown_pid_file(true)      // is optional, see `Daemonize` documentation
                .working_directory(&server_folder_) // for default behaviour.
                .redirect_dir(Some(server_folder_))
                .umask(0o022);    // Set umask, `0o027` by default.
            daemonize.start()
                .chain_err(|| "start in daemon mode")?;
        }
    }
    let store_for_http = Arc::clone(&store);
    let http_address = config.http_address.clone();
    thread::spawn(move || {
        let _http = http_server::start(&http_address, store_for_http);
    });
    let store_for_actix = Arc::clone(&store);
    let actix_address = config.actix_address.clone();
    thread::spawn(move || {
        let _actix = weaver_web::start(&actix_address, store_for_actix);
    });
    thread::spawn( move || {

    });
    let rpc = weaver_rpc::server::Server::new(&config.rpc_address, store)?;
    rpc.start();

    Ok(Server)
}

pub fn is_running(config: &ServerConfig) -> bool {
    is_listening(&config.http_address)
}

pub fn check(config: &ServerConfig) -> Result<()> {
    println!("http listening {}", is_listening(&config.http_address));
    println!("actix listening {}", is_listening(&config.actix_address));
    println!("rpc listening {}", is_listening(&config.rpc_address));
    let rpc = weaver_rpc::client::check(&config.rpc_address)?;
    println!("rpc status {}", rpc);
    Ok(())
}
