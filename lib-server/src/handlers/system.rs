/// APIs to manage the overall system.
use actix_web::{http, App, Error, State};
use app_state::AppState;

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn reload(state: State<AppState>) -> Result<String, Error> {
    let template = &state.template;
    Ok(template.reload()?)
}

pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    app.resource("/reload", |r| {
        r.method(http::Method::GET).with(reload);
    })
}
