use ::config::{file_utils, ServerRun};
use ::http_server;
use daemonize::Daemonize;
use std::fs;
use std::net::{TcpListener, ToSocketAddrs};
use std::path::PathBuf;
use std::thread;
use weaver_error::*;
use weaver_rpc;

const HTTP_ADDRESS: &'static str = "127.0.0.1:8464";
const RPC_ADDRESS: &'static str = "127.0.0.1:8465";


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
pub fn start(run: &ServerRun) -> Result<Server> {
    match run {
        &ServerRun::Foreground => {}
        &ServerRun::Daemonize => {
            let pid_file_ = pid_file()?;
            let server_folder_ = server_folder()?;
            let daemonize = Daemonize::new()
                .pid_file(pid_file_) // Every method except `new` and `start`
                .chown_pid_file(true)      // is optional, see `Daemonize` documentation
                .working_directory(&server_folder_) // for default behaviour.
                .redirect_dir(Some(server_folder_))
                .umask(0o022);    // Set umask, `0o027` by default.
            let _ = daemonize.start()
                .chain_err(|| "start in daemon mode")?;
        }
    }
    thread::spawn( || {
        let _http = http_server::start(HTTP_ADDRESS);
    });
    let rpc = weaver_rpc::server::Server::new(RPC_ADDRESS)?;
    rpc.start();

    Ok(Server)
}

pub fn is_running() -> bool {
    is_listening(HTTP_ADDRESS)
}

pub fn check() -> Result<()> {
    println!("http listening {}", is_listening(HTTP_ADDRESS));
    println!("rpc listening {}", is_listening(RPC_ADDRESS));
    let rpc = weaver_rpc::client::check(RPC_ADDRESS)?;
    println!("rpc status {}", rpc);
    Ok(())
}
