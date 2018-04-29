use futures::Future;
use grpcio::{RpcContext, ServerBuilder, UnarySink};
use proto::actions::{Epic, FormattedAction as OutputAction, History};
use proto::actions_grpc::{self, Historian};
use protobuf::repeated::RepeatedField;
use std::sync::Arc;
use weaver_db::local_api;
use weaver_db::RealStore;

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
            .and_then(|c| local_api::history(opt_name, &c)) {
            Ok(actions) => {
                let mut output = RepeatedField::new();
                for action in actions.into_iter() {
                    let mut o = OutputAction::new();
                    o.set_name(action.name);
                    o.set_id(action.id as u32);
                    o.set_kind(action.kind);
                    o.set_location(action.location.unwrap_or(String::from("")));
                    o.set_epic(action.epic.unwrap_or(String::from("")));
                    output.push(o);
                }
                reply.set_action(output);
                let f = sink.success(reply)
                    .map_err(move |err| eprintln!("Failed to reply: {:?}", err));
                ctx.spawn(f)
            }
            Err(_e) => {
                /* TODO: figure out how to fail a call.
                let status = RpcStatus::new(RpcStatusCode::Unknown, None);
                let f = sink.fail(status);
                ctx.spawn(f);
                */
            }
        };
    }
}

// Register current service with the Server Builder.
pub fn register(builder: ServerBuilder, store: Arc<RealStore>) -> ServerBuilder {
    builder.register_service(actions_grpc::create_historian(HistorianService(store)))
}
