use actix_web::http::StatusCode;
use actix_web::{error, http, App, HttpRequest, HttpResponse, Responder, State};
use asset_map::AssetMap;
use std::sync::Arc;

const FAVICON: &[u8] = include_bytes!("../../assets/favicon.ico");
const SVGS: &[u8] = include_bytes!("../../assets/inline.svg");

/// favicon handler
fn favicon(_: &HttpRequest<Arc<AssetMap>>) -> impl Responder {
    HttpResponse::build(StatusCode::OK)
        .content_type("image/x-icon")
        .body(FAVICON)
}

fn svgs(_: &HttpRequest<Arc<AssetMap>>) -> impl Responder {
    HttpResponse::build(StatusCode::OK)
        .content_type("image/svg+xml")
        .body(SVGS)
}

fn css(state: State<Arc<AssetMap>>) -> impl Responder {
    match state.asset("weaver.css") {
        Ok(css) => Ok(HttpResponse::build(StatusCode::OK)
            .content_type("text/css")
            .body(css)),
        Err(_) => Err(error::ErrorNotFound("missing css file")),
    }
}

pub(crate) fn config(app: App<Arc<AssetMap>>) -> App<Arc<AssetMap>> {
    let app = app.resource("/favicon.ico", |r| r.f(favicon));
    let app = app.resource("/inline.svg", |r| r.f(svgs));
    app.resource("weaver.css", |r| r.method(http::Method::GET).with(css))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestServer;
    use actix_web::*;

    #[test]
    fn test_svg() {
        let mut srv = TestServer::build_with_state(|| Arc::new(AssetMap::build())).start(|app| {
            app.resource("/index.html", |r| r.f(svgs));
        });

        let request = srv
            .get()
            .uri(srv.url("/index.html"))
            .finish()
            .expect("request");
        let response = srv.execute(request.send()).expect("execute send");

        assert!(response.status().is_success());
        let bytes = srv.execute(response.body()).expect("execute body");
        let data = String::from_utf8(bytes.to_vec()).expect("bytes");
        assert_eq!(&data[0..6], "<?xml ");
    }
}
