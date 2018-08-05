#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
use actix_web::http::StatusCode;
use actix_web::{App, HttpRequest, HttpResponse, Responder};

const FAVICON: &[u8] = include_bytes!("../../assets/favicon.ico");
const SVGS: &[u8] = include_bytes!("../../assets/inline.svg");

/// favicon handler
fn favicon(_: &HttpRequest<()>) -> impl Responder {
    HttpResponse::build(StatusCode::OK)
        .content_type("image/x-icon")
        .body(FAVICON)
}

fn svgs(_: &HttpRequest<()>) -> impl Responder {
    HttpResponse::build(StatusCode::OK)
        .content_type("image/svg+xml")
        .body(SVGS)
}

pub(crate) fn config(app: App<()>) -> App<()> {
    let app = app.resource("/favicon.ico", |r| r.f(favicon));
    app.resource("/inline.svg", |r| r.f(svgs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::{self, TestServer};
    use actix_web::*;

    #[test]
    fn test() {
        let mut srv = TestServer::build_with_state(|| {
            ()
        }).start(|app| {
            app.resource("/index.html", |r| r.f(svgs));
        });

        let request = srv.get().uri(srv.url("/index.html")).finish().expect("request");
        let response = srv.execute(request.send()).expect("execute send");

        assert!(response.status().is_success());
        let bytes = srv.execute(response.body()).expect("execute body");
        let data = String::from_utf8(bytes.to_vec()).expect("bytes");
        assert_eq!(&data[0..6], "<?xml ");
    }
}
