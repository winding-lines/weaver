/// Log access to a given url.
use actix_web::{http, App, Json, State};
use crate::app_state::ApiState;
use lib_db::actions2;
use lib_error::*;
use lib_goo::entities::NewAction;

#[derive(::serde::Serialize, ::serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BrowserAction {
    pub url: String,
    pub transition_type: String,
}

fn create((state, b_action): (State<ApiState>, Json<BrowserAction>)) -> Result<String> {
    let connection = state.sql.connection()?;
    let action = NewAction::build_from_url(&b_action.url, b_action.transition_type.as_str(), None)?;
    let code = actions2::insert(&connection, &action)?;
    Ok(format!("{}", code))
}

pub(crate) fn config(app: App<ApiState>) -> App<ApiState> {
    app.resource("/url", |r| {
        r.method(http::Method::POST).with(create);
    })
}
