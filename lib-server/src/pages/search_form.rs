use super::PageState;
use actix_web::{App, Error, HttpResponse, Query, State};
use lib_db::{actions2, store_policies};
use lib_goo::date;
use lib_goo::entities::lda;
use lib_index::Results;
use std::collections::HashMap;
use crate::template_engine::build_context;

// One search entry as used by the template.
#[derive(::serde::Serialize)]
struct Data<'a> {
    title: &'a str,
    url: &'a str,
    last_access: String,
    topic_ids: Vec<&'a lda::RelTopic>,
}

// One topic entry as used by the template.
#[derive(::serde::Serialize)]
struct TopicInfo {
    id: usize,
    count: usize,
    display: String,
}

// All the data used in the template
#[derive(::serde::Serialize)]
struct Datum<'a> {
    total: u64,
    matches: Vec<Data<'a>>,
    topics: Vec<TopicInfo>,
}

impl<'a> Datum<'a> {
    fn add_topic(&mut self, topic_id: usize, topic: &lda::Topic) {
        let entry: &mut TopicInfo = match self.topics.iter().position(|ref x| x.id == topic_id) {
            Some(pos) => &mut self.topics[pos],
            None => {
                let n = TopicInfo {
                    id: topic_id,
                    count: 0,
                    display: display_topic(topic),
                };
                self.topics.push(n);
                let last = self.topics.len() - 1;
                &mut self.topics[last]
            }
        };
        entry.count += 1;
    }
}

// Display the structure of a topic.
fn display_topic(topic: &lda::Topic) -> String {
    topic
        .words
        .iter()
        .map(|w| w.w.as_str())
        .collect::<Vec<&str>>()
        .join(" ")
}

/// Render the initial form or the results page, depending on the data passed in.
fn _handle(
    (state, query): (State<PageState>, Query<HashMap<String, String>>),
) -> Result<HttpResponse, Error> {
    let template = &state.template;
    let topic_store = &*state.api.topic_store;
    let mut ctx = build_context(&state.analyses);
    let rendered = if let Some(term) = query.get("term") {
        let indexer = &*state.api.indexer;

        // Fetch results from indexer
        let mut results = indexer.search(term).unwrap_or_else(|_| Results::default());

        // Process the hidden output and topics
        let connection = state.api.sql.connection()?;
        let restrictions = store_policies::Restrictions::fetch(&connection)?;

        let hidden_title = String::from("********");
        let mut datum = Datum {
            total: results.total,
            matches: Vec::with_capacity(results.matches.len()),
            topics: Vec::new(),
        };

        for result in &mut results.matches {
            let title = if !restrictions.should_display(result) {
                &hidden_title
            } else {
                &result.title
            };
            let topic_ids = if let Some(ref actual_store) = topic_store {
                if let Some(rel_topics) = actual_store.topics_for_url(&result.url) {
                    let mut out = Vec::with_capacity(rel_topics.len());
                    for rt in rel_topics {
                        let topic = actual_store.topic_at_ndx(rt.t);
                        datum.add_topic(rt.t, topic);
                        out.push(rt);
                    }
                    out
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };
            let last_access = actions2::last_access(&connection, &result.url)
                .unwrap()
                .as_ref()
                .map(|d| date::pretty_diff(date::age(d).unwrap_or(-1)))
                .unwrap_or_default();
            let data = Data {
                title,
                url: &result.url,
                last_access,
                topic_ids,
            };
            datum.matches.push(data);
        }

        datum.topics.sort_unstable_by_key(|topic| topic.count);
        datum.topics.reverse();

        ctx.insert("term", &term.to_owned());
        ctx.insert("results", &datum);
        template.render("search-results.html", &ctx)
    } else {
        ctx.insert("term", &" ".to_owned());
        template.render("search-form.html", &ctx)
    }?;
    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

fn handle(arg: (State<PageState>, Query<HashMap<String, String>>)) -> Result<HttpResponse, Error> {
    _handle(arg).map_err(|a| {
        println!("error {:?}", a);
        a
    })
}

pub(crate) fn config(app: App<PageState>) -> App<PageState> {
    app.resource("/", |r| r.with(handle))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestServer;
    use actix_web::*;
    use crate::app_state::tests::default_test;
    use crate::asset_map::AssetMap;
    use lib_db::test_helpers::SqlStoreInMemory;
    use std::sync::Arc;
    use crate::template_engine::TemplateEngine;

    use lib_goo::entities::PageContent;

    fn state() -> PageState {
        let mut s = default_test();
        s.indexer
            .add(&PageContent {
                url: "url foo".into(),
                title: "title bar".into(),
                body: "body baz".into(),
            })
            .expect("adding test PageContent");
        s.sql = Arc::new(SqlStoreInMemory::build(|_| Ok(())));
        PageState {
            api: s,
            analyses: None,
            assets: Arc::new(AssetMap::default()),
            template: Arc::new(TemplateEngine::build().unwrap()),
        }
    }

    #[test]
    fn test_search_results() {
        let mut srv = TestServer::build_with_state(|| state()).start(|app| {
            app.resource("/", |r| r.with(handle));
        });

        let request = srv
            .get()
            .uri(srv.url("/?term=1"))
            .finish()
            .expect("request");
        let response = srv.execute(request.send()).expect("execute send");

        assert!(response.status().is_success());
        let bytes = srv.execute(response.body()).expect("execute body");
        let data = String::from_utf8(bytes.to_vec()).expect("bytes");
        assert!(data.contains("title bar"));
    }
}
