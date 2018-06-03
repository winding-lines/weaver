use actix_web::{App, error, Error, HttpResponse, Query, State};
use app_state::AppState;
use std::collections::HashMap;
use tera;
use weaver_error::{Result as WResult, ResultExt};
use weaver_index::Results;

const INLINE_CSS: &str = include_str!("../../templates/inline.css");

pub fn build_tera() -> WResult<tera::Tera> {
    let mut tera = tera::Tera::default();
    tera.add_raw_templates(vec![

        ("search-form", include_str!("../../templates/search-form.html")),

        ("search-results", include_str!("../../templates/search-results.html"))
    ]).chain_err(|| "template error")?;
    Ok(tera)
}

/// Basic server check.
fn handle((state, query): (State<AppState>, Query<HashMap<String, String>>)) -> Result<HttpResponse, Error> {
    let template = state.template.as_ref()?;
    let mut ctx = tera::Context::new();
    ctx.add("inline_css", INLINE_CSS);
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

pub(crate)

fn config(app: App<AppState>) -> App<AppState> {
    app.resource("/", |r| r.with(handle))
}
