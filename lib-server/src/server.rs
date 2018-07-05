use actix_web::{App, server};
use actix_web::middleware::Logger;
use app_state::AppState;
use handlers;
use pages;
use std::sync::Arc;
use lib_db::RealStore;
use lib_error::*;
use lib_index::{Indexer, Repo};


pub struct Server {}


impl Server {
    pub fn start(addr: &str, store: Arc<RealStore>, repo: Arc<Repo>) -> Result<Server> {
        let indexer = Arc::new(Indexer::build()?);

        let s = server::new(move ||
            {
                App::with_state(AppState {
                    store: store.clone(),
                    indexer: indexer.clone(),
                    repo: repo.clone(),
                    template: pages::build_tera(),
                })
                    .middleware(Logger::new("%t %P \"%r\" %s %b %T"))
                    .configure(handlers::config)
                    .configure(pages::config)

            }
        ).bind(addr)?;
        s.run();
        Ok(Server {})
    }
}