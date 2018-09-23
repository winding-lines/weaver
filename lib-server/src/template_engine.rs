use actix_web::{error, Error};
use crate::analyses::*;
use lib_error::{Result as Wesult};
use lib_goo::config::file_utils;
use std::sync::Mutex;
use tera;
use walkdir::WalkDir;

/// Template engine providing reload functionality and more integrated error
/// loading on top of the tera engine.
pub struct TemplateEngine(Mutex<tera::Tera>);

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
            // Display a brief list of all the actions.
            ("hud.html", include_str!("../templates/hud.html")),
        ]).map_err(|_| "template error")?;

        Ok(TemplateEngine(Mutex::new(tera)))
    }

    /// Reload the template files, only in dev mode
    pub fn reload(&self) -> Wesult<String> {
        use std::path::Path;
        let mut processed = Vec::new();

        for entry in WalkDir::new(Path::new("lib-server/templates")) {
            let entry = entry.map_err(|_| "listing templates")?;
            let path = entry.path();
            if let Some(os_name) = path.file_name() {
                if let Some(name) = os_name.to_str() {
                    if entry.file_type().is_file()
                        && (name.ends_with(".html") || name.ends_with(".raw"))
                    {
                        let content = file_utils::read_content(&path)?;
                        self.0
                            .lock()
                            .unwrap()
                            .add_raw_template(name, &content)
                            .map_err(|_| "adding template")?;
                        processed.push(name.to_owned());
                    }
                }
            }
        }
        Ok(processed.join(" "))
    }

    // Pass-through the render function to the underlying engine.
    pub fn render(&self, name: &str, ctx: &tera::Context) -> Result<String, Error> {
        let lock = self
            .0
            .lock()
            .map_err(|_e| error::ErrorInternalServerError("cannot lock the rendering engine"))?;
        lock.render(name, ctx).map_err(|e| {
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
        ctx.insert("analyses", canned);
    } else {
        ctx.insert("analyses", &(Vec::new() as Vec<Analysis>));
    }
    ctx
}
