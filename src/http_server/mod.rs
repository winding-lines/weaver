use gotham;
use gotham::http::response::create_response;
use gotham::middleware::NewMiddleware;
use gotham::pipeline::*;
use gotham::pipeline::single::*;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::State;
use hyper::{Response, StatusCode};
use mime;
use weaver_error::*;
use weaver_db::{Destination, RealStore};
use std::sync::Arc;


mod store_middleware;
mod url_handler;
mod epic_handler;


#[derive(StateData)]
pub struct StoreData {
    epic: Option<String>,
    destination: Destination,
}

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
fn router<M>(store_provider: M) -> Router
    where M: NewMiddleware + Sized + Sync + Send + 'static {
    let (chain, pipelines) = single_pipeline(
        new_pipeline()
            .add(store_provider)
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
pub fn start(addr: &str, store: Arc<RealStore>) -> Result<Server> {
    print!("http on {} |", addr);
    let store_provider = store_middleware::StoreMiddleware::new(store);
    gotham::start(addr, router(store_provider));
    Ok(Server {})
}


#[cfg(test)]
mod tests {
    use gotham::handler::HandlerFuture;
    use gotham::middleware::Middleware;
    use gotham::test::TestServer;
    use serde_json;
    use super::*;

    #[derive(StateData)]
    struct TestStore {}

    #[derive(NewMiddleware, Copy, Clone, Default)]
    struct TestStoreProvider {}

    impl Middleware for TestStoreProvider {
        fn call<Chain>(self, mut state: State, chain: Chain) -> Box<HandlerFuture>
            where Chain: FnOnce(State) -> Box<HandlerFuture> + 'static
        {
            state.put(TestStore {});
            chain(state)
        }
    }

    #[test]
    fn get_product_response() {
        let test_server = TestServer::new(router(TestStoreProvider {})).unwrap();
        let response = test_server
            .client()
            .get("http://localhost/url")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);

        let body = response.read_body().unwrap();
        let product: Result<url_handler::BrowserAction> = serde_json::from_slice(&body)
            .chain_err(|| "wut");
        assert_eq!(product.is_ok(), true);
    }
}
