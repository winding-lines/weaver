extern crate actix_web;
extern crate bytes;
extern crate futures;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate weaver_db;
extern crate weaver_error;

use actix_web::{App, http, AsyncResponder, Error as ActixError, HttpMessage, HttpRequest, HttpResponse, middleware, server};
use std::sync::Arc;
use futures::prelude::*;
use futures::{Future, Stream};
use weaver_db::RealStore;
use weaver_error::*;
use std::convert;

fn hello(_req: HttpRequest<AppState>) -> &'static str {
    "Hello world!"
}

#[derive(Debug, Serialize, Deserialize)]
struct PageContent {
    title: String,
    body: String,
}

/*
impl convert::From<actix_web::error::JsonPayloadError> for Error {
    fn from(e: actix_web::error::JsonPayloadError) -> Self {
        "json error".into()
    }
}*/

fn text_index(req: HttpRequest<AppState>) -> Box<Future<Item=HttpResponse, Error = ActixError>> {
    req.json()
        .from_err()  // convert all errors into `Error`
        .and_then(|val: PageContent| {
            println!("model: {:?}", val);
            Ok(HttpResponse::Ok().json(val))  // <- send response
        })
        .responder()
}

struct AppState {
    store: Arc<RealStore>,
}

pub struct Server {}

pub fn start(addr: &str, store: Arc<RealStore>) -> Result<Server> {
    let s = server::new(move ||
        {
            App::with_state(AppState { store: store.clone() })
                .middleware(middleware::Logger::default())
                .resource("/", |r| r.f(hello))
                .resource("/text_index", |r|
                    r.method(http::Method::POST).f(text_index))
        }
    ).bind(addr)?;
    s.run();
    Ok(Server {})
}
