use super::build_context;
use actix_web::{error, App, Error, HttpResponse, Query, State};
use app_state::AppState;
use lib_db::{store_policies, topics};
use lib_index::Results;
use std::collections::HashMap;

#[derive(Serialize)]
struct Data<'a> {
    title: &'a str,
    url: &'a str,
    topics: Option<String>,
}

#[derive(Serialize)]
struct Datum<'a> {
    total: u64,
    matches: Vec<Data<'a>>,
}

/// Render the initial form or the results page, depending on the data passed in.
fn handle(
    (state, query): (State<AppState>, Query<HashMap<String, String>>),
) -> Result<HttpResponse, Error> {
    let template = state.template.as_ref()?;
    let mut ctx = build_context(&state.analyses);
    let rendered = if let Some(term) = query.get("term") {
        let indexer = &*state.indexer;

        // Fetch results from indexer
        let mut results = indexer.search(term).unwrap_or_else(|_| Results::default());

        // Process the hidden output and topics
        let connection = state.sql.connection()?;
        let restrictions = store_policies::Restrictions::fetch(&connection)?;
        let topic_store = topics::TopicStore::load()?;

        let hidden_title = String::from("********");
        let mut datum = Datum {
            total: results.total,
            matches: Vec::with_capacity(results.matches.len()),
        };

        for mut result in results.matches.iter_mut() {
            let title = if !restrictions.should_display(result) {
                &hidden_title
            } else {
                &result.title
            };
            let topics = if let Some(ref actual_store) = topic_store {
                if let Some(rel_topics) = actual_store.topics_for_url(&result.url) {
                    let mut out = String::new();
                    for rt in rel_topics {
                        let topic = actual_store.topic_at_ndx(rt.t - 1);
                        let desc = topic
                            .words
                            .iter()
                            .map(|w| w.w.as_str())
                            .collect::<Vec<&str>>()
                            .join(" ");
                        out.push_str(&format!(" {} ({:.4}) [{}]", rt.t, rt.p, desc));
                    }
                    Some(out)
                } else {
                    None
                }
            } else {
                None
            };
            let data = Data {
                title,
                url: &result.url,
                topics,
            };
            datum.matches.push(data);
        }

        ctx.add("term", &term.to_owned());
        ctx.add("results", &datum);
        template.render("search-results", &ctx)
    } else {
        ctx.add("term", &" ".to_owned());
        template.render("search-form", &ctx)
    };
    let rendered = rendered
        .map_err(|e| error::ErrorInternalServerError(format!("Template rendering {:?}", e)))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    app.resource("/", |r| r.with(handle))
}
