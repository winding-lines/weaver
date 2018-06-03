use actix_web::{App, middleware, server};
use app_state::AppState;
use handlers;
use std::sync::Arc;
use weaver_db::RealStore;
use weaver_error::*;
use weaver_index::Indexer;


pub struct Server {}


impl Server {
    pub fn start(addr: &str, store: Arc<RealStore>) -> Result<Server> {
        let indexer = Arc::new(Indexer::build()?);

        let s = server::new(move ||
            {
                App::with_state(AppState {
                    store: store.clone(),
                    indexer: indexer.clone(),
                    template: handlers::build_tera(),
                })
                    .middleware(middleware::Logger::default())
                    .configure(handlers::config)
            }
        ).bind(addr)?;
        s.run();
        Ok(Server {})
    }
}
