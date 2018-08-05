use super::build_context;
use actix_web::{error, App, Error, HttpResponse, Query, State};
use app_state::AppState;
use lib_db::actions2::{self, fetch_all};
use lib_goo::config::net::{PaginatedActions, Pagination};
use std::collections::HashMap;

/// Render the history page.
fn handle(
    (state, _query): (State<AppState>, Query<HashMap<String, String>>),
) -> Result<HttpResponse, Error> {
    let template = state.template.as_ref()?;
    let mut ctx = build_context(&state.analyses);
    ctx.add("term", &" ".to_owned());

    let connection = state.sql.connection()?;
    let pagination = Pagination {
        start: Some(0),
        length: Some(200),
    };
    let fetched = fetch_all(&connection, &pagination)?;
    let mut results = PaginatedActions {
        entries: fetched,
        total: actions2::count(&connection)?,
    };
    results.entries.reverse();
    ctx.add("results", &results);
    let rendered = template.render("history", &ctx);
    let rendered = rendered
        .map_err(|e| error::ErrorInternalServerError(format!("Template rendering {:?}", e)))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    app.resource("/history", |r| r.with(handle))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tera;
    use pages::build_tera;
    use lib_goo::entities::FormattedAction;

    #[test]
    fn test_render() {
        let mut ctx = tera::Context::new();
        let results = PaginatedActions {
            total: 14,
            entries: vec![FormattedAction::default()]
        };
        ctx.add("results", &results);
        ctx.add("inline_css", "<!-- css -->");
        ctx.add("analyses", &Vec::<String>::new());
        build_tera().unwrap().render("history", &ctx).unwrap();
    }
}
