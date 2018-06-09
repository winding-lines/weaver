use actix_web::App;
use app_state::AppState;
pub use self::search_form::build_tera;

mod search_form;
mod search_api;
mod url;
mod summary;
mod static_assets;
mod url_restrictions;


/// Configure all the handlers in the app
pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    let app = search_form::config(app);
    let app = search_api::config(app);
    let app = url::config(app);
    let app = summary::config(app);
    let app = static_assets::config(app);
    url_restrictions::config(app)
}