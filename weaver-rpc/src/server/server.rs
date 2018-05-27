use futures::Future;
use futures::sync::oneshot;
use grpcio::{self, Environment, ServerBuilder};
use std::net::ToSocketAddrs;
use std::sync::Arc;
use super::{greeter, historian};
use weaver_db::RealStore;
use weaver_error::{Result, ResultExt};


pub struct Server(grpcio::Server);


impl Server {
    pub fn new(rpc_addr: &str, store: Arc<RealStore>) -> Result<Server> {
        match rpc_addr.to_socket_addrs().map(|ref mut i| i.next()) {
            Ok(Some(a)) => {
                let env = Arc::new(Environment::new(1));
                let inner = ServerBuilder::new(env);
                let inner = greeter::register(inner);
                let inner = historian::register(inner, store);
                let inner = inner.bind(format!("{}", a.ip()), a.port())
                    .build()
                    .chain_err(|| "grpc server start")?;
                for &(ref host, port) in inner.bind_addrs() {
                    print!("rpc {}:{} |", host, port);
                }
                Ok(Server(inner))
            }
            Ok(None) | Err(_) => {
                Err("bad rpc address".into())
            }
        }
    }

    pub fn start(mut self) {

        // Start a run loop (?) Look for better API if this works.
        let (_tx, rx) = oneshot::channel::<()>();
        self.0.start();
        rx.wait().expect("run loop wait");
    }

    pub fn shutdown(mut self) -> Result<()> {
        self.0.shutdown().wait().map(|_| ()).chain_err(|| "server shutdown")
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        println!("Dropping weaver_rpc::server");
    }
}
