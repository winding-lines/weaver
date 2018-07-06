use actix_web::App;
use app_state::AppState;

mod action_api;
mod search_api;
mod url;
mod summary;
mod url_policies;


/// Configure all the handlers in the app
pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    let app = action_api::config(app);
    let app = search_api::config(app);
    let app = url::config(app);
    let app = summary::config(app);
    url_policies::config(app)
}