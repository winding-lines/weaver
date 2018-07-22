#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]

use actix_web::{http, App, HttpResponse, Json, Path, Query, State};
use app_state::AppState;
use bson::{self, Bson};
use lib_db::actions2;
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

/// Initial fetch implementation, no pagination.
/// Returns a list of actions.
fn fetch(state: State<AppState>) -> HttpResponse {
    let store = &*state.store;
    match &store
        .connection()
        .and_then(|c| actions2::fetch_all(&c, &net::Pagination::default()))
    {
        Ok(all) => HttpResponse::Ok().json(all),
        Err(e) => {
            error!("actions_api error {:?}", e);
            println!("actions_api error {:?}", e);
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

pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    let app = app.resource(net::ACTIONS2_BASE, |r| {
        r.method(http::Method::GET).with(paginated_fetch);
        r.method(http::Method::POST).with(create);
    });
    let app = app.resource(net::ACTIONS_BASE, |r| {
        r.method(http::Method::GET).with(fetch);
        r.method(http::Method::POST).with(create);
    });
    app.resource(
        &format!("{}/{{id}}{}", net::ACTIONS_BASE, net::ANNOTATIONS),
        |r| {
            r.method(http::Method::POST).with(set_annotation);
        },
    )
}
