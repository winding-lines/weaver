//! Define the html pages used by the app and the related assets.
//!
use actix_web::App;
use app_state::AppState;
use tera;
use lib_error::{Result as Wesult, ResultExt};
use ::analyses::*;

mod history;
mod search_form;
pub mod static_assets;
mod canned;

const INLINE_CSS: &str = include_str!("../../templates/inline.css");

/// Initialize the Tera template system.
pub fn build_tera() -> Wesult<tera::Tera> {
    let mut tera = tera::Tera::default();

    // Programmatically add all the templates.
    tera.add_raw_templates(vec![
        // Define the basic structure of the page.
        ("base.html", include_str!("../../templates/base.html")),
        // Display reports pre-generated on the disk.
        ("canned.raw", include_str!("../../templates/canned.raw")),
        // Search across all the documents in the repo.
        ("search-form", include_str!("../../templates/search-form.html")),
        // Display the search results.
        ("search-results", include_str!("../../templates/search-results.html")),
        // Display a lot of all the actions.
        ("history", include_str!("../../templates/history.html"))
    ]).chain_err(|| "template error")?;

    Ok(tera)
}

/// Initialize a Tera context with the expected globals.
pub fn build_context(canned: &Option<Vec<Analysis>>) -> tera::Context {


    let mut ctx = tera::Context::new();
    if let Some(canned) = canned {
        ctx.add("analyses", canned);
    } else {
        ctx.add("analyses", &(Vec::new() as Vec<Analysis>));
    }
    ctx.add("inline_css", INLINE_CSS);
    ctx
}

/// Count the number of times the configuration code is ran.
fn is_first_run() -> bool {
    use std::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
    static RUN: AtomicUsize = ATOMIC_USIZE_INIT;
    let run = RUN.fetch_add(1, Ordering::SeqCst);
    run == 1
}

/// Configure all the pages in the app
pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    let should_log = is_first_run();
    let app = canned::config(app, should_log);
    let app = search_form::config(app);
    history::config(app)
}
