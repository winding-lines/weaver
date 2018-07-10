/// Handle the pre-build analyses content.
///
use actix_web::{App, error, Error, HttpRequest, HttpResponse};
use analyses::get_analysis;
use app_state::AppState;

fn handle(req: HttpRequest<AppState>) -> Result<HttpResponse, Error> {
    match req.match_info().get("name") {
        Some(to) => {
            let content = get_analysis(to)?;
            Ok(HttpResponse::Ok().content_type("text/html").body(content))
        }
        None => Err(error::ErrorInternalServerError("bad name".to_owned())),
    }
}


pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    app.resource("/analyses/{name}", |r| r.f(handle))
}