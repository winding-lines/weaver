use actix_web::{App, HttpRequest};
use app_state::AppState;

/// Basic server check.
fn hello<T>(_req: HttpRequest<T>) -> &'static str {
    "Hello world!"
}

pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    app.resource("/", |r| r.f(hello))
}
