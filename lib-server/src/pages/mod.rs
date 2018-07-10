//! Define the html pages used by the app and the related assets.
//!
use actix_web::App;
use app_state::AppState;
use tera;
use lib_error::{Result as Wesult, ResultExt};
use ::analyses::*;

mod history;
mod search_form;
mod static_assets;
mod canned;

const INLINE_CSS: &str = include_str!("../../templates/inline.css");

/// Initialize the Tera template system.
pub fn build_tera() -> Wesult<tera::Tera> {
    let mut tera = tera::Tera::default();

    // Programmatically add all the templates.
    tera.add_raw_templates(vec![
        ("base.html", include_str!("../../templates/base.html")),
        ("search-form", include_str!("../../templates/search-form.html")),
        ("search-results", include_str!("../../templates/search-results.html")),
        ("history", include_str!("../../templates/history.html"))
    ]).chain_err(|| "template error")?;

    Ok(tera)
}

/// Initialize a Tera context with the expected globals.
pub fn build_context(canned: &Option<Vec<Analysis>>) -> tera::Context {


    let mut ctx = tera::Context::new();
    if let Some(canned) = canned {
        ctx.add("analyses", canned);
    }
    ctx.add("inline_css", INLINE_CSS);
    ctx
}

/// Configure all the pages in the app
pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    let app = canned::config(app);
    let app = search_form::config(app);
    let app = static_assets::config(app);
    history::config(app)
}
