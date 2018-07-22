#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]

use actix_web::{http, App, HttpResponse, Json, Path, Query, State};
use app_state::AppState;
use bson::{self, Bson};
use lib_ai::recommender;
use lib_db::{actions2, RealStore};
use lib_error::Result as Wesult;
use lib_error::*;
use lib_goo::config::net;
use lib_goo::entities::NewAction;
use lib_index::repo::Collection;
use lib_index::Repo;

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
    let store = &*state.store;
    actions2::insert(&store.connection()?, new_action).map(|d| format!("{}", d))
}

/// Returns a list of actions, no pagination..
fn fetch(state: State<AppState>) -> HttpResponse {
    let store = &*state.store;
    match &store
        .connection()
        .and_then(|c| actions2::fetch_all(&c, &net::Pagination::default()))
    {
        Ok(all) => HttpResponse::Ok().json(all),
        Err(e) => {
            error!("actions_api error {:?}", e);
            HttpResponse::build(http::StatusCode::INTERNAL_SERVER_ERROR).finish()
        }
    }
}

/// Maximum number of recommendations to return.
const MAX_RECS: usize = 500;

fn build_recommendations(
    store: &RealStore,
    query: &net::RecommendationQuery,
) -> Result<net::PaginatedActions> {
    let connection = store.connection()?;
    let historical = actions2::fetch_all(&connection, &net::Pagination::default())?;
    let mut recommended = recommender::recommend(&historical);

    // Fill with historical information.
    let max_recs = query.length.unwrap_or(MAX_RECS as i64);
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
    let store = &*state.store;
    match build_recommendations(store, &input) {
        Ok(paginated) => HttpResponse::Ok().json(paginated),
        Err(e) => {
            error!("recommendations_api error {:?}", e);
            HttpResponse::build(http::StatusCode::INTERNAL_SERVER_ERROR).finish()
        }
    }
}

/// Pagination enabled fetch.
/// Returns actions together with pagination meta data.
fn paginated_fetch((state, input): (State<AppState>, Query<net::Pagination>)) -> HttpResponse {
    let store = &*state.store;
    let pagination = &*input;
    debug!("Entering paginated_fetch {:?}", pagination);
    match &store
        .connection()
        .and_then(|c| actions2::fetch_all(&c, pagination).map(|all| (c, all)))
        .and_then(|(c, all)| actions2::count(&c).map(|total| (all, total)))
    {
        Ok((entries, total)) => {
            let out = net::PaginatedActions {
                entries: entries.to_vec(),
                total: *total,
            };
            HttpResponse::Ok().json(out)
        }
        Err(e) => {
            error!("actions_api error {:?}", e);
            HttpResponse::build(http::StatusCode::INTERNAL_SERVER_ERROR).finish()
        }
    }
}

/// Update the annotation for the given action.
fn set_annotation(
    (state, input, path): (State<AppState>, Json<net::Annotation>, Path<u64>),
) -> Wesult<String> {
    let store = &*state.store;
    actions2::set_annotation(&store.connection()?, *path, &input.annotation)
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

    // legacy actions
    if should_log {
        debug!("registering {}", net::ACTIONS_BASE);
    }
    let app = app.resource(net::ACTIONS_BASE, |r| {
        r.method(http::Method::GET).with(fetch);
        r.method(http::Method::POST).with(create);
    });

    // annotation setter
    let ann = format!("{}/{{id}}{}", net::ACTIONS2_BASE, net::ANNOTATIONS);
    if should_log {
        debug!("registering {}", ann);
    }
    app.resource(&ann, |r| {
        r.method(http::Method::POST).with(set_annotation);
    })
}
