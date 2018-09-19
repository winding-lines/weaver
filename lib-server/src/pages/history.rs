use super::PageState;
use actix_web::{App, Error, HttpResponse, Query, State};
use lib_db::actions2;
use lib_goo::config::net::{PaginatedActions, Pagination};
use lib_goo::entities::ActionId;
use std::collections::HashMap;
use template_engine::build_context;

/// Render the history page.
fn handle(
    (state, _query): (State<PageState>, Query<HashMap<String, String>>),
) -> Result<HttpResponse, Error> {
    let template = &state.template;
    let mut ctx = build_context(&state.analyses);
    ctx.add("term", &" ".to_owned());

    let connection = state.api.sql.connection()?;
    let count = actions2::count(&connection)? as i64;
    let pagination = Pagination {
        start: Some(count - 200),
        length: Some(200),
    };
    let mut fetched = actions2::fetch(&connection, None, &pagination)?;
    fetched.reverse();
    let results = PaginatedActions {
        entries: fetched,
        total: actions2::count(&connection)?,
        cycles: Vec::new(),
    };
    ctx.add("results", &results);
    let rendered = template.render("history.html", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

#[derive(Serialize)]
struct HudEntry {
    id: ActionId,
    when: String,
    kind: String,
    location: Option<String>,
    name: String,
}

fn hud(
    (state, query): (State<PageState>, Query<HashMap<String, String>>),
) -> Result<HttpResponse, Error> {
    let template = &state.template;
    let mut ctx = build_context(&None);
    let since_id = query.get("since").and_then(|s| {
        if let Ok(id) = s.parse::<usize>() {
            Some(ActionId::new(id))
        } else {
            None
        }
    });

    let connection = state.api.sql.connection()?;
    let count = actions2::count(&connection)? as i64;
    let pagination = Pagination {
        start: Some(count - 200),
        length: Some(200),
    };
    let fetched = actions2::fetch(&connection, None, &pagination)?;
    let mut results: Vec<HudEntry> = Vec::new();
    for action in fetched.into_iter().rev() {
        let keep = match since_id {
            Some(ref limit) => limit.is_before(&action.id),
            None => true,
        };
        if keep {
            let when = action.when.as_ref().map(|a| a.to_js()).unwrap_or_default();
            let entry = HudEntry {
                id: action.id,
                when,
                name: action.name,
                kind: action.kind,
                location: action.location,
            };
            results.push(entry);
        }
    }
    ctx.add("results", &results);
    let rendered = template.render("hud.html", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

pub(crate) fn config(app: App<PageState>) -> App<PageState> {
    let app = app.resource("/history", |r| r.with(handle));
    app.resource("/hud", |r| r.with(hud))
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
            cycles: Vec::new(),
        };
        ctx.add("results", &results);
        ctx.add("analyses", &Vec::<String>::new());
        TemplateEngine::build()
            .unwrap()
            .render("history.html", &ctx)
            .unwrap();
    }
}
