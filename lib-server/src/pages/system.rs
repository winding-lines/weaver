#![allow(clippy::needless_pass_by_value)]
use super::PageState;
/// APIs to manage the overall system.
use actix_web::{http, App, Error, State};


fn reload(state: State<PageState>) -> Result<String, Error> {
    let mut one = state.template.reload()?;
    let two = state.assets.reload()?;
    one.push_str(&two);
    Ok(one)
}

pub(crate) fn config(app: App<PageState>) -> App<PageState> {
    app.resource("/reload", |r| {
        r.method(http::Method::GET).with(reload);
    })
}
