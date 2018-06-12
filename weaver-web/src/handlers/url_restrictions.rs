use actix_web::{App, http, Json, State, Query};
use app_state::AppState;
use weaver_db::url_restrictions;
use weaver_error::*;

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UrlRestriction {
    pub url: String,
    pub kind: String,
}

fn create((state, input): (State<AppState>, Json<UrlRestriction>)) -> Result<String> {
    let store = &*state.store;
    url_restrictions::insert(&store.connection()?, &input.kind, &input.url)?;

    let indexer = &*(state.indexer);
    indexer.delete(&input.url)?;

    Ok("created".into())
}

pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    app.resource("/url_restrictions", |r| {
        r.method(http::Method::POST).with(create);
    })
}
