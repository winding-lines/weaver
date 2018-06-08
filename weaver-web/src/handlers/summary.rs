use actix_web::{App, http, State};
use app_state::AppState;


#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn summary(state: State<AppState>) -> String {
    state.indexer.search("weaver")
        .map(|r| format!("Indexed docs: {}", r.total))
        .unwrap_or_else(|_| "Index error".to_owned())
}

pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    app.resource("/summary", |r| {
        r.method(http::Method::GET).with(summary);
    })
}
