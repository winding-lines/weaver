use super::StoreData;
use futures::{future, Future, Stream};
use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::http::response::create_response;
use gotham::state::FromState;
use gotham::state::State;
use hyper::{Response, StatusCode};
use hyper::Body;
use mime;
use serde_json as json;
use weaver_db::RealStore;

#[derive(Deserialize)]
struct SetEpic {
    name: String
}

pub fn get_handler(mut state: State) -> (State, Response) {
    let res = {
        let epic = {
            let store = state.borrow_mut::<StoreData>();
            store.epic.clone().unwrap_or(String::from(""))
        };
        create_response(
            &state,
            StatusCode::Ok,
            Some((epic.into_bytes(), mime::TEXT_PLAIN)),
        )
    };

    (state, res)
}


pub fn post_handler(mut state: State) -> Box<HandlerFuture> {
    let f = Body::take_from(&mut state)
        .concat2()
        .then(move |full_body| match full_body {
            Ok(body) => {
                debug!("received url");
                let input = body.to_vec();
                let action: SetEpic = json::from_slice(&input).expect("input");
                let res = {
                    {
                        RealStore::save_epic(action.name).expect("activate epic");
                    }
                    create_response(
                        &state,
                        StatusCode::Ok,
                        None,
                    )
                };
                future::ok((state, res))
            }

            Err(e) => future::err((state, e.into_handler_error())),
        });
    Box::new(f)
}
