/// Return summary information about the stores in text format. The intent is to plug this information
/// directly in the UI.
use actix_web::{App, http, State};
use app_state::AppState;


#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn summary(state: State<AppState>) -> String {
    state.indexer.summary().unwrap_or_default()
}

pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    app.resource("/summary", |r| {
        r.method(http::Method::GET).with(summary);
    })
}
