use actix_web::{App, error, Error, HttpResponse, Query, State};
use app_state::AppState;
use std::collections::HashMap;
use super::build_context;
use weaver_db::actions2::fetch_all;
use lib_api::entities::FormattedAction;

#[derive(Serialize)]
struct Actions {
    total: usize,
    entries: Vec<FormattedAction>
}


/// Render the history page.
fn handle((state, _query): (State<AppState>, Query<HashMap<String, String>>)) -> Result<HttpResponse, Error> {
    let template = state.template.as_ref()?;
    let mut ctx = build_context();
    ctx.add("term", &" ".to_owned());

    let store = &*state.store;
    let mut entries = fetch_all(&store.connection()?)?;
    entries.reverse();
    let results = Actions {
        total: entries.len(),
        entries,
    };
    ctx.add("results", &results);
    let rendered = template
        .render("history", &ctx);
    let rendered = rendered
        .map_err(|e| error::ErrorInternalServerError(format!("Template rendering {:?}", e)))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    app.resource("/history", |r| r.with(handle))
}
