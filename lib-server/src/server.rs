use crate::analyses::load_analyses;
use crate::app_state::ApiState;
use crate::asset_map::AssetMap;
use crate::handlers;
use crate::pages;
use crate::template_engine::TemplateEngine;
use actix_web::middleware::Logger;
use actix_web::{server, App};
use lib_db::{topics, SqlStore};
use lib_error::*;
use lib_index::repo::EncryptedRepo;
use lib_index::TantivyIndexer;
use std::sync::Arc;

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
    let identity = Identity::from_pkcs12(&identity, "1234").context("ssl".into())?;
    let acceptor = TlsAcceptor::new(identity).context("acceptor")?;

    Ok(server::NativeTlsAcceptor::new(acceptor))
}

#[cfg(feature = "rust-tls")]
fn config_tls() -> Result<rustls::ServerConfig> {
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
            .context("cert open".into())?;
        cert_path.pop();
        let mut buf_cert = BufReader::new(&cert[..]);
        certs(&mut buf_cert).map_err(|e| WeaverError::from(format!("cert decode {:?}", e)))?
    };

    let mut keys = {
        cert_path.push("localhost.key");
        let mut key = vec![];
        File::open(&cert_path)?
            .read_to_end(&mut key)
            .context("key open".into())?;
        let mut buf_key = BufReader::new(&key[..]);
        pkcs8_private_keys(&mut buf_key)
            .map_err(|e| WeaverError::from(format!("key decode {:?}", e)))?
    };

    if keys.is_empty() {
        return Err("empty keys".into());
    }
    let mut config = ServerConfig::new(NoClientAuth::new());
    config
        .set_single_cert(cert_chain, keys.remove(0))
        .map_err(|e| format!("set_single_cert {:?}", e))?;

    Ok(config)
}

/// Placeholder struct for further expansion.
pub struct Server {}

impl Server {
    pub fn start(
        http_port: u16,
        https_port: u16,
        address: &str,
        base_url: String,
        store: Arc<SqlStore>,
        repo: Arc<EncryptedRepo>,
    ) -> Result<Server> {
        let indexer = Arc::new(TantivyIndexer::build()?);
        let template = Arc::new(TemplateEngine::build()?);
        let topic_store = Arc::new(topics::TopicStore::load()?);
        let asset_map = Arc::new(AssetMap::build());
        let apps_factory = move || {
            let assets_url = format!("{}/assets", base_url);
            vec![
                App::with_state(asset_map.clone())
                    .prefix(assets_url)
                    .middleware(Logger::new("%t %P \"%r\" %s %b %T"))
                    .configure(pages::static_assets::config)
                    .boxed(),
                App::with_state(ApiState {
                    sql: store.clone(),
                    indexer: indexer.clone(),
                    repo: repo.clone(),
                    topic_store: topic_store.clone(),
                })
                .prefix(format!("{}/api", base_url))
                .middleware(Logger::new("%t %P \"%r\" %s %b %T"))
                .configure(handlers::config)
                .boxed(),
                App::with_state(pages::PageState {
                    template: template.clone(),
                    assets: asset_map.clone(),
                    analyses: load_analyses().ok(),
                    api: ApiState {
                        sql: store.clone(),
                        indexer: indexer.clone(),
                        repo: repo.clone(),
                        topic_store: topic_store.clone(),
                    },
                })
                .prefix(base_url.clone())
                .middleware(Logger::new("%t %P \"%r\" %s %b %T"))
                // Add the html pages
                .configure(pages::config)
                .boxed(),
            ]
        };

        let mut s = server::new(apps_factory);
        s = s.bind(format!("{}:{}", address, http_port))?;
        #[cfg(any(feature = "tls", feature = "rust-tls"))]
        {
            match config_tls() {
                Ok(config) => {
                    ::log::info!("Initializing TLS on port {}", https_port);
                    let acceptor = server::RustlsAcceptor::with_flags(
                        config,
                        server::ServerFlags::HTTP1 | server::ServerFlags::HTTP2,
                    );
                    s = s.bind_with(format!("0.0.0.0:{}", https_port), move || acceptor.clone())?;
                }
                Err(e) => ::log::error!("Cannot start TLS {:?}", e),
            }
        }
        s.run();
        Ok(Server {})
    }
}
