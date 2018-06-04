use actix_web::{App, http, State};
use app_state::AppState;

fn summary(state: State<AppState>) -> String {
    let total = state.indexer.search("weaver")
        .map(|r| format!("Indexed docs: {}", r.total))
        .unwrap_or_else(|_| "Index error".to_owned());
    total
}

pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    app.resource("/summary", |r| {
        r.method(http::Method::GET).with(summary);
    })
}
