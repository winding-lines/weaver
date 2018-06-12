use actix_web::App;
use app_state::AppState;
use tera;
use weaver_error::{Result as Wesult, ResultExt};

mod history;
mod search_form;

const INLINE_CSS: &str = include_str!("../../templates/inline.css");

/// Initialize the Tera template system.
pub fn build_tera() -> Wesult<tera::Tera> {
    let mut tera = tera::Tera::default();

    // Programmatically add all the templates.
    tera.add_raw_templates(vec![
        ("search-form", include_str!("../../templates/search-form.html")),
        ("search-results", include_str!("../../templates/search-results.html")),
        ("history", include_str!("../../templates/history.html"))
    ]).chain_err(|| "template error")?;

    Ok(tera)
}

/// Initialize a Tera context with the expected globals.
pub fn build_context() -> tera::Context {

    let mut ctx = tera::Context::new();
    ctx.add("inline_css", INLINE_CSS);
    ctx
}

/// Configure all the pages in the app
pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    let app = search_form::config(app);
    history::config(app)
}
