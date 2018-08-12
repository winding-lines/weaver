use actix_web::{App, Error, HttpResponse, Query, State};
use app_state::AppState;
use lib_db::actions2;
use lib_goo::config::net::{PaginatedActions, Pagination};
use std::collections::HashMap;
use template_engine::build_context;

/// Render the history page.
fn handle(
    (state, _query): (State<AppState>, Query<HashMap<String, String>>),
) -> Result<HttpResponse, Error> {
    let template = &state.template;
    let mut ctx = build_context(&state.analyses);
    ctx.add("term", &" ".to_owned());

    let connection = state.sql.connection()?;
    let pagination = Pagination {
        start: Some(0),
        length: Some(200),
    };
    let fetched = actions2::fetch(&connection, None, &pagination)?;
    let mut results = PaginatedActions {
        entries: fetched,
        total: actions2::count(&connection)?,
    };
    results.entries.reverse();
    ctx.add("results", &results);
    let rendered = template.render("history.html", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    app.resource("/history", |r| r.with(handle))
}

#[cfg(test)]
mod tests {
    use super::*;
    use lib_goo::entities::FormattedAction;
    use template_engine::TemplateEngine;
    use tera;

    #[test]
    fn test_render() {
        let mut ctx = tera::Context::new();
        let results = PaginatedActions {
            total: 14,
            entries: vec![FormattedAction::default()],
        };
        ctx.add("results", &results);
        ctx.add("inline_css", "<!-- css -->");
        ctx.add("analyses", &Vec::<String>::new());
        TemplateEngine::build()
            .unwrap()
            .render("history.html", &ctx)
            .unwrap();
    }
}
