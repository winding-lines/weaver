#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
use actix_web::{App, http, HttpResponse, Json, State};
use app_state::AppState;
use lib_db::url_policies;
use lib_error::*;

/// Fetch the URL policies from the database.
/// Do not return the do_not_index entries since this may be a privacy issue.
fn fetch(state: State<AppState>) -> HttpResponse {
    let store = &*state.store;

    match store.connection().and_then(|c| url_policies::fetch_all(&c)) {
        Ok(ref mut all) => {
            debug!("url_policies {:?}", all);
            all.do_not_index.clear();
            HttpResponse::Ok().json(all)
        }
        Err(e) => {
            error!("url_policies error {:?}", e);
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
    let store = &*state.store;
    let policy = input.kind.parse::<url_policies::UrlPolicy>()?;
    debug!("marked private {}", input.url);
    url_policies::insert(&store.connection()?, &policy, &input.url)?;

    let indexer = &*(state.indexer);
    indexer.delete(&input.url)?;

    Ok("created".into())
}

/// Add our routes to the Actix server configuration.
pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    app.resource("/url_policies", |r| {
        r.method(http::Method::POST).with(create);
        r.method(http::Method::GET).with(fetch);
    })
}
