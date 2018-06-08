use actix_web::{App, http, Json, Query, State};
use app_state::AppState;
use weaver_db::url_restrictions;
use weaver_error::{Result as Wesult};

#[derive(Debug, Serialize, Deserialize)]
struct PageContent {
    url: String,
    title: String,
    body: String,
}

// API used by the Chrome extension to upload content to be indexed.
#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn _create((state, input): (State<AppState>, Json<PageContent>)) -> Wesult<String> {
    let store = &*state.store;
    let connection = store.connection()?;
    let url_restrictions = url_restrictions::fetch_all(&connection)?;
    if !url_restrictions.should_index(&input.url) {
        return Ok("{}".into());
    }

    let indexer = &*(state.indexer);
    let id = indexer.add(&input.url, &input.title, &input.body)?;

    Ok(format!("{}", id))
}

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
fn create(data: (State<AppState>, Json<PageContent>)) -> String {
   _create(data).unwrap_or_else(|_| "error".into())
}

# [derive(Deserialize)]
struct SearchQuery {
    term: String,
}

// API used to make a query and download the matches.
fn search((state, query): (State<AppState>, Query<SearchQuery>)) -> String {
    let indexer = &*state.indexer;

    indexer.search(&query.term)
        .map(|f|
            f.matches.iter()
                .map(|d| format!("{} {}\n", d.0, d.1))
                .fold(String::new(), |mut a, n| {
                    a.push_str(&n);
                    a
                }))
    .unwrap_or_else(|_| String::new())
}

pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    app.resource("/search", |r| {
        r.method(http::Method::GET).with(search);
        r.method(http::Method::POST).with(create);
    })
}
