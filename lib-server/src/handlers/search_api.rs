#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]

use actix_web::{App, http, HttpResponse, Json, Query, State};
use app_state::AppState;
use lib_ai::normalize;
use lib_goo::entities::PageContent;
use lib_index::repo::Collection;
use lib_db::store_policies;
use lib_error::{Result as Wesult, ResultExt};
use bincode;


#[derive(Debug, Serialize, Deserialize)]
struct PageStatus {
    is_indexed: bool,
    summary: Option<String>,
}

fn _create((state, mut input): (State<AppState>, Json<PageContent>)) -> Wesult<PageStatus> {
    let store = &*state.store;
    let repo = &*state.repo;

    let connection = store.connection()?;
    input.url = normalize::normalize_url(&input.url)?.into_owned();

    let url_restrictions = store_policies::Restrictions::fetch(&connection)?;
    if !url_restrictions.should_index(&input) {
        return Ok(PageStatus { is_indexed: false, summary: state.indexer.summary() });
    }

    let serialized = bincode::serialize(&*input)
        .chain_err(|| "serializing for the repo")?;
    repo.add(&Collection(PageContent::collection_name().into()), &serialized)?;
    let indexer = &*(state.indexer);
    let _id = indexer.add(&input)?;

    Ok(PageStatus { is_indexed: true, summary: state.indexer.summary() })
}

// API used by the Chrome extension to upload content to be indexed.
fn create(data: (State<AppState>, Json<PageContent>)) -> HttpResponse {
    match _create(data) {
        Ok(ps) => HttpResponse::Ok().json(ps),
        Err(e) => {
            error!("search_api error {:?}", e);
            HttpResponse::build(http::StatusCode::INTERNAL_SERVER_ERROR).finish()
        }
    }
}

#[derive(Deserialize)]
struct SearchQuery {
    term: String,
}

// API used to make a query and download the matches.
fn search((state, query): (State<AppState>, Query<SearchQuery>)) -> String {
    let indexer = &*state.indexer;

    indexer.search(&query.term)
        .map(|f|
            f.matches.iter()
                .map(|d| format!("{} {}\n", d.url, d.title))
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
