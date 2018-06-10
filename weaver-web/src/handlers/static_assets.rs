#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
use actix_web::{App, HttpRequest, HttpResponse, Result};
use actix_web::http::StatusCode;
use app_state::AppState;

const FAVICON: &[u8] = include_bytes!("../../assets/favicon.ico");
const SVGS: &[u8] = include_bytes!("../../assets/inline.svg");


/// favicon handler
fn favicon(_req: HttpRequest<AppState>) -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("image/x-icon")
        .body(FAVICON))
}

fn svgs(_req: HttpRequest<AppState>) -> Result<HttpResponse> {
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("image/svg+xml")
        .body(SVGS))
}

pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    let app = app.resource("/favicon.ico", |r|
        r.f(favicon),
    );
    app.resource("/assets/inline.svg", |r|
        r.f(svgs),
    )
}
