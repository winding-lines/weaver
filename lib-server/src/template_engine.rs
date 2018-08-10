use actix_web::{error, Error};
use analyses::*;
use lib_error::{Result as Wesult, ResultExt};
use lib_goo::config::file_utils;
use std::cell::RefCell;
use tera;
use walkdir::WalkDir;

const INLINE_CSS: &str = include_str!("../templates/inline.css");

/// Template engine providing reload functionality and more integrated error
/// loading on top of the tera engine.
pub struct TemplateEngine(RefCell<tera::Tera>);

impl TemplateEngine {
    /// Initialize the Tera template system.
    pub fn build() -> Wesult<Self> {
        let mut tera = tera::Tera::default();

        // Programmatically add all the templates.
        tera.add_raw_templates(vec![
            // Define the basic structure of the page.
            ("base.html", include_str!("../templates/base.html")),
            // Display reports pre-generated on the disk.
            ("canned.raw", include_str!("../templates/canned.raw")),
            // Search across all the documents in the repo.
            (
                "search-form.html",
                include_str!("../templates/search-form.html"),
            ),
            // Display the search results.
            (
                "search-results.html",
                include_str!("../templates/search-results.html"),
            ),
            // Display a lot of all the actions.
            ("history.html", include_str!("../templates/history.html")),
        ]).chain_err(|| "template error")?;

        Ok(TemplateEngine(RefCell::new(tera)))
    }

    /// Reload the template files, only in dev mode
    pub fn reload(&self) -> Wesult<()> {
        use std::path::Path;

        for entry in WalkDir::new(Path::new("lib-server/templates")) {
            let entry = entry.chain_err(|| "listing templates")?;
            let path = entry.path();
            if let Some(os_name) = path.file_name() {
                if let Some(name) = os_name.to_str() {
                    if entry.file_type().is_file()
                        && (name.ends_with(".html") || name.ends_with(".raw"))
                    {
                        let content = file_utils::read_content(&path)?;
                        self.0.borrow_mut().add_raw_template(name, &content).chain_err(|| "adding template")?;
                    }
                }
            }
        }
        Ok(())
    }

    // Pass-through the render function to the underlying engine.
    pub fn render(&self, name: &str, ctx: &tera::Context) -> Result<String, Error> {
        self.0.borrow().render(name, ctx).map_err(|e| {
            error::ErrorInternalServerError(format!(
                "Failed rendering {} with error: {:?}",
                name, e
            ))
        })
    }
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
