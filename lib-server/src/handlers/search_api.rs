#![cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]

use actix_web::{http, App, HttpResponse, Json, Query, State};
use crate::app_state::ApiState;
use bincode;
use lib_goo::normalize;
use lib_db::{store_policies, pages};
use lib_error::{Result as Wesult};
use lib_goo::entities::PageContent;
use lib_index::repo::Collection;

#[derive(Debug, Serialize, Deserialize)]
struct PageStatus {
    is_indexed: bool,
    summary: Option<String>,
}

fn _create((state, mut input): (State<ApiState>, Json<PageContent>)) -> Wesult<PageStatus> {
    let repo = &*state.repo;

    let connection = state.sql.connection()?;
    input.url = normalize::normalize_url(&input.url)?.into_owned();

    let url_restrictions = store_policies::Restrictions::fetch(&connection)?;
    if !url_restrictions.should_index(&input) {
        return Ok(PageStatus {
            is_indexed: false,
            summary: state.indexer.summary(),
        });
    }

    let serialized = bincode::serialize(&*input).map_err(|_| "serializing for the repo")?;
    repo.add(
        &Collection(PageContent::collection_name().into()),
        &serialized,
    )?;
    let indexer = &*(state.indexer);
    let _id = indexer.add(&input)?;

    let _page_id = pages::fetch_or_create_id(&connection, &input.url, Some(&input.title))?;

    Ok(PageStatus {
        is_indexed: true,
        summary: state.indexer.summary(),
    })
}

// API used by the Chrome extension to upload content to be indexed.
fn create(data: (State<ApiState>, Json<PageContent>)) -> HttpResponse {
    match _create(data) {
        Ok(ps) => HttpResponse::Ok().json(ps),
        Err(e) => {
            error!("search_api error {:?}", e);
            HttpResponse::build(http::StatusCode::INTERNAL_SERVER_ERROR).finish()
        }
    }
}

// Query parameters coming from the client
#[derive(Deserialize)]
struct SearchQuery {
    term: String,
}

// API used to make a query and download the matches.
fn search((state, query): (State<ApiState>, Query<SearchQuery>)) -> String {
    let indexer = &*state.indexer;

    indexer
        .search(&query.term)
        .map(|f| {
            f.matches
                .iter()
                .map(|d| format!("{} {}\n", d.url, d.title))
                .fold(String::new(), |mut a, n| {
                    a.push_str(&n);
                    a
                })
        })
        .unwrap_or_else(|_| String::new())
}

pub(crate) fn config(app: App<ApiState>) -> App<ApiState> {
    app.resource("/search", |r| {
        r.method(http::Method::GET).with(search);
        r.method(http::Method::POST).with(create);
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestServer;
    use actix_web::*;
    use crate::app_state::tests::default_test;

    fn state() -> ApiState {
        let s = default_test();
        s.indexer
            .add(&PageContent {
                url: "url foo".into(),
                title: "title bar".into(),
                body: "body baz".into(),
            })
            .expect("adding test PageContent");
        s
    }

    #[test]
    fn test_resource() {
        let mut srv = TestServer::build_with_state(|| state()).start(|app| {
            app.resource("/search", |r| {
                r.method(http::Method::GET).with(search);
                r.method(http::Method::POST).with(create);
            });
        });

        let request = srv
            .get()
            .uri(srv.url("/search?term=1"))
            .finish()
            .expect("request");
        let response = srv.execute(request.send()).expect("execute send");

        assert!(response.status().is_success());
        let bytes = srv.execute(response.body()).expect("execute body");
        let data = String::from_utf8(bytes.to_vec()).expect("bytes");
        assert_eq!(&data, "url foo title bar\n");
    }
}
