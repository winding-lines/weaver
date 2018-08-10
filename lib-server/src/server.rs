use actix_web::middleware::Logger;
use actix_web::{server, App};
use analyses::load_analyses;
use app_state::AppState;
use handlers;
use lib_db::SqlStore;
use lib_error::*;
use lib_index::repo::EncryptedRepo;
use lib_index::TantivyIndexer;
use pages;
use std::sync::Arc;
use template_engine::TemplateEngine;

/// Placeholder struct for further expansion.
pub struct Server {}

impl Server {
    pub fn start(addr: &str, store: Arc<SqlStore>, repo: Arc<EncryptedRepo>) -> Result<Server> {
        let indexer = Arc::new(TantivyIndexer::build()?);

        let s = server::new(move || {
            vec![
                App::new()
                    .prefix("/assets/")
                    .middleware(Logger::new("%t %P \"%r\" %s %b %T"))
                    .configure(pages::static_assets::config)
                    .boxed(),
                App::with_state(AppState {
                    sql: store.clone(),
                    indexer: indexer.clone(),
                    repo: repo.clone(),
                    template: TemplateEngine::build(),
                    analyses: load_analyses().ok()
                })
                    .prefix("/api/")
                    .middleware(Logger::new("%t %P \"%r\" %s %b %T"))
                    // Add the API entry points.
                    .configure(handlers::config)
                    .boxed(),
                App::with_state(AppState {
                    sql: store.clone(),
                    indexer: indexer.clone(),
                    repo: repo.clone(),
                    template: TemplateEngine::build(),
                    analyses: load_analyses().ok()
                })
                    .middleware(Logger::new("%t %P \"%r\" %s %b %T"))
                    // Add the API entry points.
                    .configure(handlers::config_obsolete)
                    // Add the html pages
                    .configure(pages::config)
                    .boxed(),
            ]
        }).bind(addr)?;
        s.run();
        Ok(Server {})
    }
}
