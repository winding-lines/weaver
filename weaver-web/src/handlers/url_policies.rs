use actix_web::{App, http, HttpResponse, Json, State};
use app_state::AppState;
use weaver_db::url_policies;
use weaver_error::*;

/// List of URls that diverge from the main processing policy.
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UrlRestriction {
    pub url: String,
    pub kind: String,
}


fn fetch(state: State<AppState>) -> HttpResponse {
    let store = &*state.store;

    match store.connection().and_then(|c| url_policies::fetch_all(&c)) {
        Ok(ref mut all) => {
            all.do_not_index.clear();
            HttpResponse::Ok().json(all)
        }
        Err(_) => HttpResponse::build(http::StatusCode::INTERNAL_SERVER_ERROR).finish(),
    }
}

fn create((state, input): (State<AppState>, Json<UrlRestriction>)) -> Result<String> {
    let store = &*state.store;
    let policy = input.kind.parse::<url_policies::UrlPolicy>()?;
    url_policies::insert(&store.connection()?, &policy, &input.url)?;

    let indexer = &*(state.indexer);
    indexer.delete(&input.url)?;

    Ok("created".into())
}

pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    app.resource("/url_policies", |r| {
        r.method(http::Method::POST).with(create);
        r.method(http::Method::GET).with(fetch);
    })
}
