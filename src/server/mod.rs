use ::errors::*;
use gotham;
use gotham::http::response::create_response;
use gotham::pipeline::*;
use gotham::pipeline::single::*;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::State;
use hyper::{Response, StatusCode};
use mime;


mod store_middleware;
mod url_handler;
mod epic_handler;

fn index(state: State) -> (State, Response) {
    let res = create_response(
        &state,
        StatusCode::Ok,
        Some((String::from("Hello Router!").into_bytes(), mime::TEXT_PLAIN)),
    );

    (state, res)
}

/// Create a `Router`
///
fn router() -> Router {
    let (chain, pipelines) = single_pipeline(
        new_pipeline()
            .add(store_middleware::StoreMiddleware)
            .build()
    );

    build_router(chain, pipelines, |route| {
        route.get("/url").to(url_handler::get_handler);
        route.post("/url").to(url_handler::post_handler);
        route.get("/epic").to(epic_handler::get_handler);
        route.post("/epic").to(epic_handler::post_handler);
        route.get("/").to(index);
    })
}

pub struct Server {}

/// Start a server and use a `Router` to dispatch requests
pub fn start() -> Result<Server> {
    let addr = "127.0.0.1:8464";
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router());
    Ok(Server {})
}


#[cfg(test)]
mod tests {

    use gotham::test::TestServer;
    use serde_json;
    use super::*;
    #[test]
    fn get_product_response() {
        let test_server = TestServer::new(router()).unwrap();
        let response = test_server
            .client()
            .get("http://localhost/url")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);

        let body = response.read_body().unwrap();
        let expected_product = url_handler::BrowserAction {
            url: "t-shirt".to_string(),
        };
        let expected_body = serde_json::to_string(&expected_product).expect("serialized product");
        assert_eq!(&body[..], expected_body.as_bytes());
    }
}
