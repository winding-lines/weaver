#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
use actix_web::{http, App, HttpResponse, Json, State};
use app_state::AppState;
use lib_db::store_policies;
use lib_db::url_restrictions;
use lib_error::*;

#[derive(Default, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Restrictions {
    pub do_not_log: Vec<String>,
    // Note: this field is used directly on the client (chrome extension) since the user can
    // force index any page.
    pub do_index: Vec<String>,
}

fn build_urls(all: &[store_policies::DocumentMatcher]) -> Vec<String> {
    all.iter()
        .filter(|ref a| a.url.is_some())
        .map(|ref a| a.url.as_ref().unwrap().as_str().to_owned())
        .collect()
}

/// Fetch the URL policies from the database.
/// Do not return the do_not_index entries since this may be a privacy issue.
fn fetch(state: State<AppState>) -> HttpResponse {
    match state
        .sql
        .connection()
        .and_then(|c| store_policies::Restrictions::fetch(&c))
    {
        Ok(ref mut all) => {
            let out = Restrictions {
                do_not_log: build_urls(&all.do_not_log),
                do_index: build_urls(&all.do_index),
            };
            debug!("store_policies {:?}", all);
            all.do_not_index.clear();
            HttpResponse::Ok().json(out)
        }
        Err(e) => {
            error!("store_policies error {:?}", e);
            HttpResponse::build(http::StatusCode::INTERNAL_SERVER_ERROR).finish()
        }
    }
}

/// Data shape when requesting to add a new URL Policy.
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UrlRestriction {
    pub url: String,
    pub kind: String,
}

/// Process a new URL policy from the client:
///  - save in the db
///  - remove any entry from the text search index.
fn create((state, input): (State<AppState>, Json<UrlRestriction>)) -> Result<String> {
    let connection = state.sql.connection()?;
    let policy = input.kind.parse::<url_restrictions::StorePolicy>()?;
    debug!("marked private {}", input.url);
    url_restrictions::insert(
        &connection,
        url_restrictions::UrlRestriction::with_url(&policy, &input.url),
    )?;

    let indexer = &*(state.indexer);
    indexer.delete(&input.url)?;

    Ok("created".into())
}

/// Add our routes to the Actix server configuration.
pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    app.resource("/store_policies", |r| {
        r.method(http::Method::POST).with(create);
        r.method(http::Method::GET).with(fetch);
    })
}
