use actix_web::{App, middleware, server};
use app_state::AppState;
use handlers;
use std::sync::Arc;
use tera;
use weaver_db::RealStore;
use weaver_error::*;
use weaver_index::Indexer;


pub struct Server {}

fn build_tera() -> Result<tera::Tera> {
    let mut tera = tera::Tera::default();
    tera.add_raw_templates(vec![
        ("hello", "<html><title>Weaver</title><body>Hello world!</body></html>")
    ]).chain_err(|| "template error")?;
    Ok(tera)
}

impl Server {
    pub fn start(addr: &str, store: Arc<RealStore>) -> Result<Server> {
        let indexer = Arc::new(Indexer::build()?);

        let s = server::new(move ||
            {
                App::with_state(AppState {
                    store: store.clone(),
                    indexer: indexer.clone(),
                    template: build_tera(),
                })
                    .middleware(middleware::Logger::default())
                    .configure(handlers::config)
            }
        ).bind(addr)?;
        s.run();
        Ok(Server {})
    }
}
