#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]

use actix_web::{http, App, HttpResponse, Json, Path, Query, State};
use app_state::AppState;
use bson::{self, Bson};
use lib_ai::recommender;
use lib_db::{actions2, Connection};
use lib_error::Result as Wesult;
use lib_error::*;
use lib_goo::config::net;
use lib_goo::entities::NewAction;
use lib_index::repo::Collection;
use lib_index::repo::Repo;

// Wrap into a bson envelope and save into the repo.
fn save_to_repo(repo: &Repo, new_action: &NewAction) -> Result<String> {
    debug!("Saving to repo");
    let data = bson::to_bson(new_action).chain_err(|| "create document for new_action")?;
    let mut document = bson::Document::new();
    document.insert("data", data);
    document.insert("type", Bson::String("NewAction".into()));
    document.insert("version", Bson::String(NewAction::version().into()));
    let mut out = Vec::<u8>::new();
    bson::encode_document(&mut out, &document).chain_err(|| "encode new_action")?;

    repo.add(&Collection(NewAction::collection_name().into()), &out)
}

/// Create a new action.
fn create((state, new_action): (State<AppState>, Json<NewAction>)) -> Wesult<String> {
    debug!("Entering create in action_api");
    let repo = &*state.repo;
    let new_action = &*new_action;
    save_to_repo(repo, new_action)?;

    debug!("Saving to db");
    actions2::insert(&state.sql.connection()?, new_action).map(|d| format!("{}", d))
}

/// Maximum number of recommendations to return.
const MAX_RECS: usize = 500;

fn build_recommendations(
    connection: &Connection,
    query: &net::RecommendationQuery,
) -> Result<net::PaginatedActions> {
    let mut historical = actions2::fetch_all(&connection, &net::Pagination::default())?;
    let mut recommended = recommender::recommend(&historical);

    // Fill with historical information.
    let max_recs = query.length.unwrap_or(MAX_RECS as i64);
    historical.reverse();
    let fill = ((max_recs as i64) - (recommended.len() as i64)) as usize;
    recommended.extend_from_slice(&historical[0..fill]);
    let count = actions2::count(&connection)?;
    Ok(net::PaginatedActions {
        entries: recommended,
        total: count,
    })
}

fn recommendations(
    (state, input): (State<AppState>, Query<net::RecommendationQuery>),
) -> HttpResponse {
    match state
        .sql
        .connection()
        .and_then(|c| build_recommendations(&c, &input))
    {
        Ok(paginated) => HttpResponse::Ok().json(paginated),
        Err(e) => {
            let msg = format!("recommendations_api error {:?}", e);
            error!("{}", msg);
            HttpResponse::build(http::StatusCode::INTERNAL_SERVER_ERROR).body(msg)
        }
    }
}

/// Pagination enabled fetch.
/// Returns actions together with pagination meta data.
fn paginated_fetch((state, input): (State<AppState>, Query<net::Pagination>)) -> HttpResponse {
    let pagination = &*input;
    debug!("Entering paginated_fetch {:?}", pagination);
    match state
        .sql
        .connection()
        .and_then(|c| actions2::fetch_all(&c, pagination).map(|all| (c, all)))
        .and_then(|(c, all)| actions2::count(&c).map(|total| (all, total)))
    {
        Ok((entries, total)) => {
            let out = net::PaginatedActions {
                entries: entries.to_vec(),
                total: total,
            };
            HttpResponse::Ok().json(out)
        }
        Err(e) => {
            let msg = format!("actions_api error {:?}", e);
            error!("{}", msg);
            HttpResponse::build(http::StatusCode::INTERNAL_SERVER_ERROR).body(msg)
        }
    }
}

/// Update the annotation for the given action.
fn set_annotation(
    (state, input, path): (State<AppState>, Json<net::Annotation>, Path<u64>),
) -> Wesult<String> {
    actions2::set_annotation(&state.sql.connection()?, *path, &input.annotation)
        .map(|d| format!("{}", d))
}

// Register the routes with the application.
pub(crate) fn config(app: App<AppState>, should_log: bool) -> App<AppState> {
    // v2 actions
    if should_log {
        debug!("registering {}", net::ACTIONS2_BASE);
    }
    let app = app.resource(net::ACTIONS2_BASE, |r| {
        r.method(http::Method::GET).with(paginated_fetch);
        r.method(http::Method::POST).with(create);
    });

    // recommendations
    let recs = format!("{}{}", net::ACTIONS2_BASE, net::RECOMMENDATIONS);
    if should_log {
        debug!("registering {}", recs);
    }
    let app = app.resource(&recs, |r| r.method(http::Method::GET).with(recommendations));

    // annotation setter
    let ann = format!("{}/{{id}}{}", net::ACTIONS2_BASE, net::ANNOTATIONS);
    if should_log {
        debug!("registering {}", ann);
    }
    app.resource(&ann, |r| {
        r.method(http::Method::POST).with(set_annotation);
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestServer;
    use actix_web::*;
    use app_state::tests::default_test;
    use lib_db::actions2;
    use lib_db::test_helpers::SqlStoreInMemory;
    use lib_goo::entities::NewAction;
    use std::sync::Arc;

    fn state() -> AppState {
        let mut s = default_test();
        s.sql = Arc::new(SqlStoreInMemory);
        let one = NewAction {
            command: "foo".into(),
            ..NewAction::default()
        };
        actions2::insert(&s.sql.connection().unwrap(), &one).expect("insert test new action");
        s
    }

    #[test]
    fn test_paginated_search() {
        let mut srv = TestServer::build_with_state(|| state()).start(|app| {
            app.resource(net::ACTIONS2_BASE, |r| {
                r.method(http::Method::GET).with(paginated_fetch);
                r.method(http::Method::POST).with(create);
            });
        });

        let request = srv
            .get()
            .uri(srv.url(net::ACTIONS2_BASE))
            .finish()
            .expect("request");
        let response = srv.execute(request.send()).expect("execute send");
        let bytes = srv.execute(response.body()).expect("execute body");
        let data = String::from_utf8(bytes.to_vec()).expect("bytes");

        // println!("response {:?} {}", response, data);
        assert!(response.status().is_success());
        assert_eq!(&data, "{\"entries\":[],\"total\":0}");
    }
}
