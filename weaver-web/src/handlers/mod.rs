use actix_web::App;
use app_state::AppState;

mod search_form;
mod search_api;
mod url;


pub use self::search_form::build_tera;

/// Configure all the handlers in the app
pub(crate) fn config(app:App<AppState>) -> App<AppState> {
    let a1 = search_form::config(app);
    let a2 = search_api::config(a1);
    url::config(a2)
}