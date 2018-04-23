extern crate grpcio;

use grpcio::{ChannelBuilder, EnvBuilder};
use proto::hello::HelloRequest;
use proto::hello_grpc::GreeterClient;
use std::sync::Arc;
use weaver_error::{Result, ResultExt};


pub fn check(rpc_addr: &str) -> Result<bool> {
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(rpc_addr);
    let client = GreeterClient::new(ch);

    let request = HelloRequest::new();
    debug!("calling say_hello");
    let reply = client.say_hello(&request).chain_err(|| "rpc check")?;
    debug!("request {:?} and got message {}", request, reply.get_message());
    Ok(true)
}