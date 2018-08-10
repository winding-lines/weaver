use actix_web::App;
use app_state::AppState;

mod action_api;
mod search_api;
mod url;
mod summary;
mod url_policies;
mod system;

/// Count the number of times the configuration code is ran.
fn is_first_run() -> bool {
    use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
    static RUN: AtomicUsize = ATOMIC_USIZE_INIT;
    let run = RUN.fetch_add(1, Ordering::SeqCst);
    run == 1
}
/// Configure all the handlers in the app
pub(crate) fn config_obsolete(app: App<AppState>) -> App<AppState> {
    let should_log = is_first_run();
    let app = action_api::config(app, should_log);
    let app = search_api::config(app);
    let app = url::config(app);
    let app = summary::config(app);
    url_policies::config(app)
}

/// Configure all the handlers in the app.
/// This is used to migrate to the new /api prefix.
pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    system::config(app)
}