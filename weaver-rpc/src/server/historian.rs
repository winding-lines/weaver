//! Server side implementation of the Historian API.


use futures::Future;
use grpcio::{RpcContext, ServerBuilder, UnarySink};
use proto::actions::*;
use proto::actions_grpc::{self, Historian};
use protobuf::repeated::RepeatedField;
use std::sync::Arc;
use weaver_db::{RealStore, actions2, epics, entities};

#[derive(Clone)]
struct HistorianService(Arc<RealStore>);

impl Historian for HistorianService {
    fn list(&self, ctx: RpcContext, req: Epic, sink: UnarySink<History>) {
        let mut reply = History::new();
        let name = req.name;
        let opt_name = if name.is_empty() {
            None
        } else {
            Some(name)
        };
        match self.0.connection()

            .and_then(|c| actions2::fetch_all(&c)) {
            Ok(actions) => {
                let mut output = RepeatedField::new();
                for action in actions.into_iter() {
                    let mut o = Action::new();
                    o.set_name(action.name);
                    o.set_id(action.id as u64);
                    o.set_kind(action.kind);
                    o.set_location(action.location.unwrap_or(String::from("")));
                    o.set_epic(action.epic.unwrap_or(String::from("")));
                    o.set_annotation(action.annotation.unwrap_or(String::from("")));
                    output.push(o);
                }
                reply.set_action(output);
                let f = sink.success(reply)
                    .map_err(move |err| eprintln!("Failed to reply: {:?}", err));
                ctx.spawn(f)
            }
            Err(e) => {
                /* TODO: figure out how to fail a call.
                let status = RpcStatus::new(RpcStatusCode::Unknown, None);
                let f = sink.fail(status);
                ctx.spawn(f);
                */
                eprintln!("Failed to fetch actions {:?}", e);
            }
        };
    }

    fn add(&self, ctx: RpcContext, req: CreateAction, sink: UnarySink<ActionId>) {

        match self.0.connection()
            .and_then(|c| {
                let new_action = entities::NewAction {
                    executed: String::from(req.get_executed()),
                    kind: req.get_kind(),
                    command: req.get_command(),
                    location: Some(req.get_location()),
                    epic: Some(req.get_epic()),
                    host: String::from(req.get_host()),
                };
                actions2::insert(&c, new_action)
            }) {
            Ok(id) => {
                let mut output = ActionId::new();
                output.set_id(id);
                let f = sink.success(output)
                    .map_err(move |err| eprintln!("Failed to reply: {:?}", err));
                ctx.spawn(f);
            }
            Err(e) => {
                eprintln!("Failed to insert action: {:?}", e);
            }
        }
    }

    fn fetch_epics(&self, ctx: RpcContext, _req: Epic, sink: UnarySink<Epics>) {
        let mut reply = Epics::new();
        match self.0.connection()
            .and_then(|c| epics::fetch_all(&c)) {
            Ok(epics) => {
                let mut output = RepeatedField::new();
                for epic in epics.into_iter() {
                    output.push(epic);
                }
                reply.set_name(output);
                let f = sink.success(reply)
                    .map_err(move |err| eprintln!("Failed to reply: {:?}", err));
                ctx.spawn(f)
            }
            Err(e) => {
                /* TODO: figure out how to fail a call.
                let status = RpcStatus::new(RpcStatusCode::Unknown, None);
                let f = sink.fail(status);
                ctx.spawn(f);
                */
                eprintln!("Failed to fetch actions {:?}", e);
            }
        };    }

    fn set_annotation(&self, ctx: RpcContext, req: Annotation, sink: UnarySink<ActionId>) {
        match self.0.connection()
            .and_then(|c| {
                actions2::set_annotation(&c, req.id, &req.annotation)
            }) {
            Ok(id) => {
                let mut output = ActionId::new();
                output.set_id(id);
                let f = sink.success(output)
                    .map_err(move |err| eprintln!("Failed to reply: {:?}", err));
                ctx.spawn(f);
            }
            Err(e) => {
                eprintln!("Failed to insert action: {:?}", e);
            }
        }
    }
}

// Register current service with the Server Builder.
pub fn register(builder: ServerBuilder, store: Arc<RealStore>) -> ServerBuilder {
    builder.register_service(actions_grpc::create_historian(HistorianService(store)))
}
