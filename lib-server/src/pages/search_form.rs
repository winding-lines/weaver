use actix_web::{App, error, Error, HttpResponse, Query, State};
use app_state::AppState;
use std::collections::HashMap;
use super::build_context;
use lib_index::Results;

/// Render the initial form or the results page, depending on the data passed in.
fn handle((state, query): (State<AppState>, Query<HashMap<String, String>>)) -> Result<HttpResponse, Error> {
    let template = state.template.as_ref()?;
    let mut ctx = build_context(&state.analyses);
    let rendered = if let Some(term) = query.get("term") {
        let indexer = &*state.indexer;
        let results = indexer.search(term).unwrap_or_else(|_| Results::default());
        ctx.add("term", &term.to_owned());
        ctx.add("results", &results);
        template
            .render("search-results", &ctx)
    } else {
        ctx.add("term", &" ".to_owned());
        template
            .render("search-form", &ctx)
    };
    let rendered = rendered
        .map_err(|e| error::ErrorInternalServerError(format!("Template rendering {:?}", e)))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    app.resource("/", |r| r.with(handle))
}
