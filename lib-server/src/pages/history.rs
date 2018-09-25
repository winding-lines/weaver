use super::PageState;
use actix_web::{App, Error, HttpResponse, Query, State};
use lib_ai::compact;
use lib_db::actions2;
use lib_goo::config::net::{PaginatedActions, Pagination};
use lib_goo::entities::ActionId;
use std::collections::HashMap;
use crate::template_engine::build_context;
use std::time::Instant;

/// Render the history page.
fn handle(
    (state, _query): (State<PageState>, Query<HashMap<String, String>>),
) -> Result<HttpResponse, Error> {
    let template = &state.template;
    let mut ctx = build_context(&state.analyses);
    ctx.insert("term", &" ".to_owned());

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
    ctx.insert("results", &results);
    let rendered = template.render("history.html", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

#[derive(Serialize)]
struct HudEntry {
    id: ActionId,
    when: String,
    kind: String,
    location: String,
    name: String,
}
#[derive(Deserialize)]
struct HudQuery {
    term: Option<String>,
    since_id: Option<usize>,
}

fn hud((state, query): (State<PageState>, Query<HudQuery>)) -> Result<HttpResponse, Error> {
    let connection = state.api.sql.connection()?;
    let count = actions2::count(&connection)? as i64;
    let term = query.term.as_ref().and_then(|a| if a.is_empty() {None} else {Some(&**a)} );
    let pagination = if term.is_some() {
        Pagination {
            start: None,
            length: None,
        }
    } else {
        Pagination {
            start: Some(count - 200),
            length: Some(200),
        }
    };
    let fetch_start = Instant::now();
    let mut fetched = actions2::fetch(&connection, term, &pagination)?;
    let duration = fetch_start.elapsed();
    info!(
        "fetched {} actions for term {:?} in {}.{}",
        fetched.len(),
        query.term,
        duration.as_secs(),
        duration.subsec_millis(),
    );
    let compact_start = Instant::now();
    let cycles = compact::extract_cycles(&fetched, 4);
    compact::decycle(&mut fetched, &cycles);
    let duration = compact_start.elapsed();
    info!("after compacting got {} actions in {}.{}", 
        fetched.len(), duration.as_secs(), duration.subsec_millis());

    // Put in the shape expected by the template.
    let serialization_start = Instant::now();
    let since_id = query.since_id.map(|s| ActionId::new(s));
    let mut results: Vec<HudEntry> = Vec::new();
    for action in fetched.into_iter().rev() {
        let keep = match since_id {
            Some(ref limit) => limit.is_before(&action.id),
            None => true,
        };
        if keep {
            let when = action.when.as_ref().map(|a| a.to_js()).unwrap_or_default();
            let location = action.location.unwrap_or(action.name.clone());
            let entry = HudEntry {
                id: action.id,
                when,
                name: action.name,
                kind: action.kind,
                location,
            };
            results.push(entry);
        }
    }
    let duration = serialization_start.elapsed();
    info!("serialization {}.{}", 
        duration.as_secs(), duration.subsec_millis());

    // Render the output.
    let template_start = Instant::now();
    let template = &state.template;
    let mut ctx = build_context(&None);
    ctx.insert("results", &results);
    ctx.insert("term", &query.term);
    let rendered = template.render("hud.html", &ctx)?;
    let duration = template_start.elapsed();
    info!("template render {}.{}", 
        duration.as_secs(), duration.subsec_millis());

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
    use crate::template_engine::TemplateEngine;
    use tera;

    #[test]
    fn test_render() {
        let mut ctx = tera::Context::new();
        let results = PaginatedActions {
            total: 14,
            entries: vec![FormattedAction::default()],
            cycles: Vec::new(),
        };
        ctx.insert("results", &results);
        ctx.insert("analyses", &Vec::<String>::new());
        TemplateEngine::build()
            .unwrap()
            .render("history.html", &ctx)
            .unwrap();
    }
}
