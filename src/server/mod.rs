use ::errors::*;
use daemonize::Daemonize;
use gotham;
use gotham::http::response::create_response;
use gotham::middleware::{NewMiddleware, Middleware};
use gotham::pipeline::*;
use gotham::pipeline::single::*;
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::State;
use hyper::{Response, StatusCode};
use mime;
use std::fs;
use std::net::{TcpListener, ToSocketAddrs};
use std::path::PathBuf;
use super::config::{file_utils, ServerRun};


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

fn server_folder() -> Result<PathBuf> {
    file_utils::app_folder().and_then(|mut path| {
        path.push("server");
        if !path.exists() {
            fs::create_dir(&path).chain_err(|| "create server folder")?;
        }
        Ok(path)
    })
}

fn pid_file() -> Result<PathBuf> {
    server_folder().map(|mut s| {
        s.push("server.pid");

        s
    })
}

const SERVER_ADDRESS: &'static str = "127.0.0.1:8464";

pub fn is_running() -> bool {
    let addr = match SERVER_ADDRESS.to_socket_addrs().map(|ref mut i| i.next()) {
        Ok(Some(a)) => a,
        Ok(_) => panic!("unable to resolve listener address"),
        Err(_) => panic!("unable to parse listener address"),
    };

    match TcpListener::bind(addr) {
        Ok(listener) => {
            // We were able to bind to the address => no server is listening.
            drop(listener);
            false
        }
        Err(_) => {
            debug!("Error binding to {}, assume the server is running.", SERVER_ADDRESS);
            true
        }
    }
}

/// Start a server and use a `Router` to dispatch requests
pub fn start(run: &ServerRun) -> Result<Server> {
    let addr = "127.0.0.1:8464";
    println!("Listening for requests at http://{}", addr);
    let store_provider = store_middleware::StoreMiddleware;
    match run {
        &ServerRun::Foreground => {
            gotham::start(addr, router(store_provider));
        }
        &ServerRun::Daemonize => {
            let pid_file_ = pid_file()?;
            let server_folder_ = server_folder()?;
            let daemonize = Daemonize::new()
                .pid_file(pid_file_) // Every method except `new` and `start`
                .chown_pid_file(true)      // is optional, see `Daemonize` documentation
                .working_directory(&server_folder_) // for default behaviour.
                .redirect_dir(Some(server_folder_))
                .umask(0o022);    // Set umask, `0o027` by default.
            let _ = daemonize.start()
                .chain_err(|| "start in daemon mode")?;
            gotham::start(addr, router(store_provider));
        }
    }
    Ok(Server {})
}


#[cfg(test)]
mod tests {
    use gotham::handler::HandlerFuture;
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
