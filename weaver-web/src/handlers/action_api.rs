#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]

use actix_web::{App, http, HttpResponse, Json, Path, Query, State};
use app_state::AppState;
use weaver_db::actions2;
use weaver_db::config::net;
use weaver_db::entities::NewAction;
use weaver_error::Result as Wesult;


fn create((state, new_action): (State<AppState>, Json<NewAction>)) -> Wesult<String> {
    let store = &*state.store;
    actions2::insert(&store.connection()?, &*new_action)
        .map(|d| format!("{}", d))
}

fn search((state, _input): (State<AppState>, Query<net::Pagination>)) -> HttpResponse {
    let store = &*state.store;
    match &store.connection().and_then(|c| actions2::fetch_all(&c)) {
        Ok(all) => HttpResponse::Ok().json(all),
        Err(_) => HttpResponse::build(http::StatusCode::INTERNAL_SERVER_ERROR).finish(),
    }
}

fn set_annotation((state, input, path): (State<AppState>, Json<net::Annotation>, Path<u64>)) -> Wesult<String> {
    let store = &*state.store;
    actions2::set_annotation(&store.connection()?, *path, &input.annotation)
        .map(|d| format!("{}", d))
}

pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    let app = app.resource(net::ACTIONS_BASE, |r| {
        r.method(http::Method::GET).with(search);
        r.method(http::Method::POST).with(create);
    });
    app.resource("/actions/{id}/annotations", |r| {
        r.method(http::Method::GET).with(set_annotation);
    })
}
