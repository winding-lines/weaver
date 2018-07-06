/// Log access to a given url.

use actix_web::{App, http, Json, State};
use app_state::AppState;
use lib_db::actions2;
use lib_error::*;
use lib_goo::entities::NewAction;

#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BrowserAction {
    pub url: String,
    pub transition_type: String,
}

fn create((state, b_action): (State<AppState>, Json<BrowserAction>)) -> Result<String> {
    let store = &*state.store;
    let action = NewAction::build_from_url(&b_action.url, b_action.transition_type.as_str(), None)?;
    let code = actions2::insert(&store.connection()?, &action)?;
    Ok(format!("{}", code))
}


pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    app.resource("/url", |r| {
        r.method(http::Method::POST).with(create);
    })
}
