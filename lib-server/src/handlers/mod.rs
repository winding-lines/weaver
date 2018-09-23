use actix_web::App;
use crate::app_state::ApiState;

mod action_api;
mod search_api;
mod summary;
mod url;
mod url_policies;

/// Count the number of times the configuration code is ran.
fn is_first_run() -> bool {
    use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
    static RUN: AtomicUsize = ATOMIC_USIZE_INIT;
    let run = RUN.fetch_add(1, Ordering::SeqCst);
    run == 1
}

/// Configure all the handlers in the app.
/// This is used to migrate to the new /api prefix.
pub(crate) fn config(app: App<ApiState>) -> App<ApiState> {
    let should_log = is_first_run();
    let app = summary::config(app);
    let app = url_policies::config(app);
    let app = search_api::config(app);
    let app = url::config(app);
    action_api::config(app, should_log)
}
