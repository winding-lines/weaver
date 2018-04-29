extern crate grpcio;

use grpcio::{ChannelBuilder, EnvBuilder};
use proto::actions_grpc::HistorianClient;
use proto::hello::HelloRequest;
use proto::hello_grpc::GreeterClient;
use std::sync::Arc;
use weaver_db::entities::FormattedAction;
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

pub fn history<T>(epic: Option<T>, rpc_addr: &str) -> Result<Vec<FormattedAction>>
where T: Into<String> {
    use proto::actions::Epic;

    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(rpc_addr);
    let client = HistorianClient::new(ch);

    let mut request = Epic::new();
    if let Some(n) = epic {
        request.set_name(n.into());
    }
    let reply = client.list(&request).chain_err(|| "rpc history")?;

    let mut actions = Vec::<FormattedAction>::new();
    for action in reply.action.into_iter() {
        actions.push(FormattedAction {
            annotation: Some(action.annotation),
            id: action.id as usize,
            epic: Some(action.epic),
            kind: action.kind,
            name: action.name,
            location: Some(action.location),
        });
    }
    Ok(actions)
}