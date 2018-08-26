use actix_web::middleware::Logger;
use actix_web::{server, App};
use analyses::load_analyses;
use app_state::AppState;
use handlers;
use lib_db::{topics, SqlStore};
use lib_error::*;
use lib_index::repo::EncryptedRepo;
use lib_index::TantivyIndexer;
use pages;
use std::sync::Arc;
use template_engine::TemplateEngine;

#[cfg(feature = "tls")]
fn config_tls() -> Result<server::NativeTlsAcceptor> {
    use lib_goo::config::file_utils;
    use native_tls::{Identity, TlsAcceptor};
    use std::fs::File;
    use std::io::Read;

    let mut cert_path = file_utils::app_folder()?;
    cert_path.push("server");
    cert_path.push("localhost.pfx");
    let mut identity = vec![];
    File::open(&cert_path)?
        .read_to_end(&mut identity)
        .map_err(|e| format!("cert open {:?}", e))?;
    let identity = Identity::from_pkcs12(&identity, "1234").map_err(|e| format!("ssl {:?}", e))?;
    let acceptor = TlsAcceptor::new(identity).map_err(|e| format!("acceptor {:?}", e))?;

    Ok(server::NativeTlsAcceptor::new(acceptor))
}

#[cfg(feature = "rust-tls")]
fn config_tls() -> Result<server::RustlsAcceptor> {
    use lib_goo::config::file_utils;
    use rustls::internal::pemfile::{certs, pkcs8_private_keys};
    use rustls::{NoClientAuth, ServerConfig};
    use std::fs::File;
    use std::io::BufReader;
    use std::io::Read;

    let mut cert_path = file_utils::app_folder()?;
    cert_path.push("server");

    let cert_chain = {
        cert_path.push("localhost.crt");
        let mut cert = vec![];
        File::open(&cert_path)?
            .read_to_end(&mut cert)
            .map_err(|e| format!("cert open {:?}", e))?;
        cert_path.pop();
        let mut buf_cert = BufReader::new(&cert[..]);
        certs(&mut buf_cert).map_err(|e| format!("cert decode {:?}", e))?
    };

    let mut keys = {
        cert_path.push("localhost.key");
        let mut key = vec![];
        File::open(&cert_path)?
            .read_to_end(&mut key)
            .map_err(|e| format!("key open {:?}", e))?;
        let mut buf_key = BufReader::new(&key[..]);
        pkcs8_private_keys(&mut buf_key).map_err(|e| format!("key decode {:?}", e))?
    };

    if keys.len() == 0 {
        return Err("empty keys".into());
    }
    let mut config = ServerConfig::new(NoClientAuth::new());
    config
        .set_single_cert(cert_chain, keys.remove(0))
        .map_err(|e| format!("set_single_cert {:?}", e))?;

    let acceptor = server::RustlsAcceptor::new(config);

    Ok(acceptor)
}

/// Placeholder struct for further expansion.
pub struct Server {}

impl Server {
    pub fn start(
        http_port: u16,
        https_port: u16,
        store: Arc<SqlStore>,
        repo: Arc<EncryptedRepo>,
    ) -> Result<Server> {
        let indexer = Arc::new(TantivyIndexer::build()?);
        let template = Arc::new(TemplateEngine::build()?);
        let topic_store = Arc::new(topics::TopicStore::load()?);
        let apps_factory = move || {
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
                    template: template.clone(),
                    analyses: load_analyses().ok(),
                    topic_store: topic_store.clone(),
                }).prefix("/api/")
                    .middleware(Logger::new("%t %P \"%r\" %s %b %T"))
                    .configure(handlers::config)
                    .boxed(),
                App::with_state(AppState {
                    sql: store.clone(),
                    indexer: indexer.clone(),
                    repo: repo.clone(),
                    template: template.clone(),
                    analyses: load_analyses().ok(),
                    topic_store: topic_store.clone(),
                })
                    .middleware(Logger::new("%t %P \"%r\" %s %b %T"))
                    // Add the API entry points.
                    .configure(handlers::config_obsolete)
                    // Add the html pages
                    .configure(pages::config)
                    .boxed(),
            ]
        };

        let mut s = server::new(apps_factory);
        s = s.bind(format!("127.0.0.1:{}", http_port))?;
        #[cfg(any(feature = "tls", feature = "rust-tls"))]
        {
            match config_tls() {
                Ok(acceptor) => {
                    info!("Initializing TLS on port {}", https_port);
                    s = s.bind_with(format!("0.0.0.0:{}", https_port), acceptor)?;
                }
                Err(e) => error!("Cannot start TLS {:?}", e),
            }
        }
        s.run();
        Ok(Server {})
    }
}
