/// Handle the pre-build analyses content.
///
use actix_web::{error, App, Error, HttpRequest, HttpResponse, State};
use analyses::get_analysis;
use app_state::AppState;
use template_engine::build_context;

fn handle((req, state): (HttpRequest<AppState>, State<AppState>)) -> Result<HttpResponse, Error> {
    match req.match_info().get("name") {
        Some(to) => {
            let content = get_analysis(to)?;
            let template = &state.template;
            let mut ctx = build_context(&state.analyses);
            ctx.add("report", to);
            ctx.add("content", &content);
            let rendered = template.render("canned.raw", &ctx)?;
            Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
        }
        None => Err(error::ErrorInternalServerError("bad name".to_owned())),
    }
}

pub(crate) fn config(app: App<AppState>, should_log: bool) -> App<AppState> {
    let url = "/analyses/{name}";
    if should_log {
        debug!("registering {}", url);
    }
    app.resource(url, |r| r.with(handle))
}
