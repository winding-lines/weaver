/// Return summary information about the stores in text format. The intent is to plug this information
/// directly in the UI.
use actix_web::{http, App, State};
use app_state::ApiState;

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn summary(state: State<ApiState>) -> String {
    state.indexer.summary().unwrap_or_default()
}

pub(crate) fn config(app: App<ApiState>) -> App<ApiState> {
    app.resource("/summary", |r| {
        r.method(http::Method::GET).with(summary);
    })
}
