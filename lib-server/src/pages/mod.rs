//! Define the html pages used by the app and the related assets.
//!
use actix_web::App;
use crate::analyses::Analysis;
use crate::app_state::ApiState;
use crate::asset_map::AssetMap;
use std::sync::Arc;
use crate::template_engine::TemplateEngine;

mod canned;
mod history;
mod search_form;
pub mod static_assets;
mod system;

pub(crate) struct PageState {
    pub analyses: Option<Vec<Analysis>>,
    pub template: Arc<TemplateEngine>,
    pub assets: Arc<AssetMap>,
    pub api: ApiState,
}

/// Count the number of times the configuration code is ran.
fn is_first_run() -> bool {
    use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};
    static RUN: AtomicUsize = ATOMIC_USIZE_INIT;
    let run = RUN.fetch_add(1, Ordering::SeqCst);
    run == 1
}

/// Configure all the pages in the app
pub(crate) fn config(app: App<PageState>) -> App<PageState> {
    let should_log = is_first_run();
    let app = canned::config(app, should_log);
    let app = search_form::config(app);
    let app = system::config(app);
    history::config(app)
}
