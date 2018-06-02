use actix_web::{App, State, HttpResponse, error, Error};
use app_state::AppState;
use tera;

/// Basic server check.
fn hello(state: State<AppState>) ->  Result<HttpResponse, Error> {
    let template = state.template.as_ref()
        .map_err(|_| error::ErrorInternalServerError("Template initialization"))?;
    let rendered = template
        .render("hello", &tera::Context::new())
        .map_err(|_| error::ErrorInternalServerError("Template rendering"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    app.resource("/", |r| r.with(hello))
}
