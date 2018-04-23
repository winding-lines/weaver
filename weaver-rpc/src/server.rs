extern crate futures;
extern crate grpcio;

use futures::Future;
use futures::sync::oneshot;
use grpcio::{Environment, RpcContext, ServerBuilder, UnarySink};
use proto::hello::{HelloReply, HelloRequest};
use proto::hello_grpc::{self, Greeter};
use std::net::ToSocketAddrs;
use std::sync::Arc;
use weaver_error::{Result, ResultExt};

#[derive(Clone)]
struct GreeterService;

impl Greeter for GreeterService {
    fn say_hello(&self, ctx: RpcContext, req: HelloRequest, sink: UnarySink<HelloReply>) {
        let mut reply = HelloReply::new();
        reply.set_message(format!("Hello back {}", req.get_name()));
        let f = sink.success(reply)
            .map_err(move |err| eprintln!("Failed to reply: {:?}", err));
        ctx.spawn(f)
    }
}

pub struct Server(grpcio::Server);

impl Server {
    pub fn new(rpc_addr: &str) -> Result<Server> {
        match rpc_addr.to_socket_addrs().map(|ref mut i| i.next()) {
            Ok(Some(a)) => {
                let env = Arc::new(Environment::new(1));
                let service = hello_grpc::create_greeter(GreeterService);
                let inner = ServerBuilder::new(env)
                    .register_service(service)
                    .bind(format!("{}",a.ip()), a.port())
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
        let _ = rx.wait().expect("run loop wait");
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
