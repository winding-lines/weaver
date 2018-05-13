extern crate grpcio;

use grpcio::{ChannelBuilder, EnvBuilder};
use proto::actions_grpc::HistorianClient;
use proto::hello::HelloRequest;
use proto::hello_grpc::GreeterClient;
use std::sync::Arc;
use weaver_db::config;
use weaver_db::entities::{FormattedAction, NewAction};
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

pub fn history(rpc_addr: &str, env: &config::Environment) -> Result<Vec<FormattedAction>> {
    use proto::actions::Epic;

    let client = {
        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(rpc_addr);
        HistorianClient::new(ch)
    };

    let mut request = Epic::new();
    if let Some(n) = env.epic() {
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

pub fn add(rpc_addr: &str, req: NewAction) -> Result<u64> {
    use proto::actions::CreateAction;

    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(rpc_addr);
    let client = HistorianClient::new(ch);
    let mut new_action = CreateAction::new();
    new_action.set_command(req.command.into());
    new_action.set_kind( req.kind.into());
    new_action.set_location( req.location.unwrap_or("").into());
    new_action.set_epic( req.epic.unwrap_or("").into());
    new_action.set_host( req.host);
    new_action.set_executed( req.executed);
    client.add(&new_action)
        .map(|c| c.id)
        .chain_err(|| "rpc add")
}

pub fn last_url(_rpc_addr: &str) -> Result<Option<(String, String)>> {
   unimplemented!()
}

pub fn fetch_epics(rpc_addr: &str) -> Result<Vec<String>> {
    use proto::actions::Epic;

    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(rpc_addr);
    let client = HistorianClient::new(ch);
    let epic = Epic::new();

    client.fetch_epics(&epic)
        .map(|c| c.name.into_vec())
        .chain_err(|| "rpc fetch_epics")
}

pub fn set_annotation(rpc_addr: &str, id: u64, content: &str) -> Result<u64> {
    use proto::actions::Annotation;

    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(rpc_addr);
    let client = HistorianClient::new(ch);

    let mut req = Annotation::new();
    req.set_id(id);
    req.set_annotation(String::from(content));
    client.set_annotation(&req)
        .map(|a| a.id)
        .chain_err(|| "set annotation")


}
