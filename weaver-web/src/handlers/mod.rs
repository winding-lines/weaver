use actix_web::App;
use app_state::AppState;

mod hello;
mod text_index;

/// Configure all the handlers in the app
pub(crate) fn config(app:App<AppState>) -> App<AppState> {
    let a1 = hello::config(app);
    text_index::config(a1)
}