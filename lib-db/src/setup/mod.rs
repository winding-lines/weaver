//! Populate the database with initial data.

use db::url_restrictions::{self, UrlRestriction};
use lib_error::*;
use lib_goo::config::file_utils;
use serde_json as json;
use Connection;

// Populate the database with the URL that should not be logged
fn do_not_log_urls(connection: &Connection) -> Result<()> {
    let sites = vec![
        // social
        "linkedin\\.com",
        "snapchat\\.com",
        "facebook\\.com",
        "qz\\.com",
        "reddit\\.com",
        "twitter\\.com",
        "\\Wt\\.co\\W",
        // news
        "\\Wnews\\.",
        "forbes\\.com",
        "electrek\\.com",
        "macrumours\\.com",
        "\\Wnytimes\\.com",
        "\\Wnpr\\.org",
        "\\Wzdnet\\.com",
        "ycombinator\\.com",
        "\\Wcbc.ca",
        // travel
        "expedia\\.com",
        "getthere\\.net",
        // financial"
        "paypal\\.",
        "wellsfargo\\.",
        "citibank\\.",
        "chase\\.",
        // auth
        "ServiceLogin",
        "okta\\.com",
        "oauth2",
        "\\Wlogin\\.",
        // content
        "imgur\\.",
        "gfycat\\.",
        // comm
        "bluejeans\\.com",
        // commerce
        "\\.ebay\\.",
        "www\\.amazon\\.com",
        "localhost:8888",
        "localhost:8466",
        "localhost",
    ];
    for u in sites {
        let ur = UrlRestriction::with_url(&url_restrictions::StorePolicy::NoLog, u);
        url_restrictions::insert(connection, ur)?;
    }
    Ok(())
}

// Populate the database with the sites that we should index.
fn do_index(connection: &Connection) -> Result<()> {
    let sites = vec![
        "https://docs.google.com/*",
        "https://*.sharepoint.com/*",
        "https://*.wikipedia.org/*",
        "https://*.apache.org/*",
        "https://*.statsdirect.com/*",
        "https://developer.mozilla.org/*",
        "https://iwww.corp.linkedin.com/*",
        "https://developer.chrome.com/*",
        "https://*.stackoverflow.com/*",
        "https://*.css-tricks.com/*",
        "https://*.readthedocs.io/*",
    ];
    for u in sites {
        let ur = UrlRestriction::with_url(&url_restrictions::StorePolicy::DoIndex, u);
        url_restrictions::insert(connection, ur)?;
    }
    Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserContent {
    restrictions: Vec<url_restrictions::UrlRestriction>,
}

fn user_defined(connection: &Connection) -> Result<()> {
    let mut input = file_utils::app_folder()?;
    input.push("user-data");
    input.push("user-content.json");
    if input.exists() {
        let content = file_utils::read_content(&input)?;
        let uc: UserContent = json::from_str(&content).map_err(|_| WeaverError::Generic("reading user content".into()))?;
        for r in uc.restrictions {
            println!("creating user-content {:?}", r);
            url_restrictions::insert(&connection, r)?;
        }
    }
    Ok(())
}

// Populate all the default data in the database.
pub fn populate_data(connection: &Connection) -> Result<()> {
    do_not_log_urls(connection)?;
    do_index(connection)?;
    user_defined(connection)?;
    Ok(())
}
