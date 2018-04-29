use ::weaver_db::local_api;
use futures::{future, Future, Stream};
use gotham::handler::{HandlerFuture, IntoHandlerError, IntoResponse};
use gotham::http::response::create_response;
use gotham::state::FromState;
use gotham::state::State;
use hyper::{Response, StatusCode};
use hyper::{Body, Chunk};
use mime;
use serde_json as json;
use super::StoreData;
use weaver_error::*;


#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BrowserAction {
    pub url: String,
    pub transition_type: String,
}

/// Implements `gotham::handler::IntoResponse` trait for `Product`
///
/// `IntoResponse` represents a type which can be converted to a response. This trait is used in
/// converting the return type of a function into a response.
///
/// This trait implementation uses the Serde project when generating responses. You don't need to
/// know about Serde in order to understand the response that is being created here but if you're
/// interested you can learn more at `http://serde.rs`.
impl IntoResponse for BrowserAction {
    fn into_response(self, state: &State) -> Response {
        create_response(
            state,
            StatusCode::Ok,
            Some((
                json::to_string(&self)
                    .expect("serialized product")
                    .into_bytes(),
                mime::APPLICATION_JSON,
            )),
        )
    }
}

/// Function to handle the `GET` requests coming to `/url`
///
/// Note that this function returns a `(State, Product)` instead of the usual `(State, Response)`.
/// As we've implemented `IntoResponse` above Gotham will correctly handle this and call our
/// `into_response` method when appropriate.
pub fn get_handler(mut state: State) -> (State, BrowserAction) {
    let last = {
        // Leaking test setup in here, need to figure out better dependeny injection.
        if let Some(store) = state.try_borrow_mut::<StoreData>() {
            local_api::last_url(&store.connection)
                .unwrap_or(None)
        } else {
            None
        }
    };

    let product = last.map(|l| BrowserAction {
        url: l.0,
        transition_type: l.1,
    }).unwrap_or_else(|| BrowserAction::default());

    (state, product)
}

fn process_post(body: Chunk, store: &StoreData) -> Result<String> {
    let input = body.to_vec();
    let action: BrowserAction = json::from_slice(&input).expect("input");
    let epic = &store.epic;
    let code = local_api::add_url_action(&store.connection, &action.url, action.transition_type.as_str(), epic.as_ref().map(String::as_str))?;
    Ok(format!("{}", code))
}

pub fn post_handler(mut state: State) -> Box<HandlerFuture> {
    let f = Body::take_from(&mut state)
        .concat2()
        .then(move |full_body| match full_body {
            Ok(body) => {
                debug!("received url");
                match process_post(body, state.borrow::<StoreData>()) {
                    Ok(out) => {
                        let res = create_response(
                            &state,
                            StatusCode::Ok,
                            Some((out.into_bytes(), mime::TEXT_PLAIN)),
                        );

                        future::ok((state, res))
                    }
                    Err(e) => future::err((state, e.into_handler_error()))
                }
            }

            Err(e) => future::err((state, e.into_handler_error())),
        });
    Box::new(f)
}

