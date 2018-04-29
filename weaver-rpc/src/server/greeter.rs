use grpcio::{RpcContext, ServerBuilder, UnarySink};
use proto::hello::{HelloReply, HelloRequest};
use proto::hello_grpc::{self, Greeter};
use futures::Future;

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

// Register current service with the Server Builder.
pub fn register(builder: ServerBuilder) -> ServerBuilder {
    builder.register_service(hello_grpc::create_greeter(GreeterService))
}

