use actix_web::{App, http, Json, Query, State};
use app_state::AppState;

#[derive(Debug, Serialize, Deserialize)]
struct PageContent {
    url: String,
    title: String,
    body: String,
}

fn create(data: (State<AppState>, Json<PageContent>)) -> String {
    let indexer = &*(data.0.indexer);
    indexer.add(&data.1.url, &data.1.title, &data.1.body)
        .map(|id| format!("{}", id))
        .unwrap_or_else(|_| String::new())
}

#[derive(Deserialize)]
struct Search {
    term: String,
}

fn search(data: (State<AppState>, Query<Search>)) -> String {
    let indexer = &*(data.0.indexer);
    indexer.search(&data.1.term)
        .map(|f|
            f.iter()
                .map(|d| format!("{} {}\n", d.0, d.1))
                .fold(String::new(), |mut a, n| {
                    a.push_str(&n);
                    a
                }))
    .unwrap_or_else(|_| String::new())
}

pub(crate) fn config(app: App<AppState>) -> App<AppState> {
    app.resource("/text_index", |r| {
        r.method(http::Method::GET).with(search);
        r.method(http::Method::POST).with(create);
    })
}
