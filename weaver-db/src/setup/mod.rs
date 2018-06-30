//! Populate the database with initial data.
use ::Connection;
use ::db::url_policies;
use weaver_error::*;

// Populate the database with the URL that should not be logged
fn do_not_log_urls(connection: &Connection) -> Result<()> {
    let sites = vec![
        // social
        "linkedin\\.com", "snapchat\\.com", "facebook\\.com",
        "qz\\.com", "reddit\\.com", "twitter\\.com", "\\Wt\\.co\\W",
        // news
        "\\Wnews\\.", "forbes\\.com", "electrek\\.com", "macrumours\\.com",
        "\\Wnytimes\\.com", "\\Wnpr\\.org", "\\Wzdnet\\.com", "ycombinator\\.com", "\\Wcbc.ca",
        // travel
        "expedia\\.com", "getthere\\.net",
        // financial"
        "paypal\\.", "wellsfargo\\.", "citibank\\.", "chase\\.",
        // auth
        "ServiceLogin", "okta\\.com", "oauth2", "\\Wlogin\\.",
        // content
        "imgur\\.", "gfycat\\.",
        // comm
        "bluejeans\\.com",
        // commerce
        "\\.ebay\\.", "www\\.amazon\\.com"
    ];
    for u in sites {
        url_policies::insert(connection, &url_policies::UrlPolicy::NoLog, u)?;
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
        "https://*.readthedocs.io/*"
    ];
    for u in sites {
        url_policies::insert(connection, &url_policies::UrlPolicy::DoIndex, u)?;
    }
    Ok(())
}

// Populate all the default data in the database.
pub fn populate_data(connection: &Connection) -> Result<()> {
    do_not_log_urls(connection)?;
    do_index(connection)?;
    Ok(())
}