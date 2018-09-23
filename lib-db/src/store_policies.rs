use crate::db::url_restrictions::{self, StorePolicy, UrlRestriction};
use lib_error::Result as WResult;
use lib_goo::entities::PageContent;
use regex::Regex;
use std::convert::From;
use Connection;

// Describe a document matcher to identify documents that belong to a certain class.
#[derive(Debug)]
pub struct DocumentMatcher {
    pub url: Option<Regex>,
    pub title: Option<Regex>,
    pub body: Option<Regex>,
}

impl<'a> From<&'a UrlRestriction> for DocumentMatcher {
    fn from(ur: &UrlRestriction) -> Self {
        DocumentMatcher {
            url: Regex::new(&ur.url_expr).ok(),
            title: ur.title_match.as_ref().and_then(|a| Regex::new(&a).ok()),
            body: ur.body_match.as_ref().and_then(|a| Regex::new(&a).ok()),
        }
    }
}

impl DocumentMatcher {
    #[allow(dead_code)]
    fn with_url(url: &str) -> WResult<DocumentMatcher> {
        let url = Regex::new(url)?;
        Ok(DocumentMatcher {
            url: Some(url),
            title: None,
            body: None,
        })
    }

    // Matches a document if all of non optional pieces are matching.
    fn matches(&self, page_content: &PageContent) -> bool {
        let url_m = self.url.as_ref().map(|r| r.is_match(&page_content.url));
        let body_m = self.body.as_ref().map(|r| r.is_match(&page_content.body));
        let title_m = self.title.as_ref().map(|r| r.is_match(&page_content.title));
        // if all are either Some(true) or None then we match
        url_m.unwrap_or(true) && body_m.unwrap_or(true) && title_m.unwrap_or(true)
    }

    fn matches_any(page_content: &PageContent, restrictions: &[DocumentMatcher]) -> bool {
        restrictions.iter().any(|r| r.matches(page_content))
    }
}

// Structure used to manipulate in memory all the restrictions and also to communicate
// them with the client.
#[derive(Default, Debug)]
pub struct Restrictions {
    pub do_not_log: Vec<DocumentMatcher>,
    pub do_not_index: Vec<DocumentMatcher>,
    // Note: this field is used directly on the client (chrome extension) since the user can
    // force index any page.
    pub do_index: Vec<DocumentMatcher>,

    // Do not display documents matching this list of matchers.
    pub hidden: Vec<DocumentMatcher>,
}

// Check if the incoming URL matches against the pre-defined restriction.
fn matches_restriction(url: &str, restriction: &Regex) -> bool {
    restriction.is_match(url)
}

// Check if the incoming URL matches against any of the pre-defined restrictions
fn matches_any_restriction(url: &str, restrictions: &[DocumentMatcher]) -> bool {
    for restriction in restrictions {
        if let Some(ref restriction) = restriction.url {
            if matches_restriction(url, &restriction) {
                return true;
            }
        }
    }
    false
}

impl Restrictions {
    // Check if there are any restrictions against indexing this URL.
    // Note that from the UI the user can force the index of any page
    // so we do not check against the do_index field.
    pub fn should_index_url(&self, url: &str) -> bool {
        !matches_any_restriction(url, &self.do_not_index)
            && !matches_any_restriction(url, &self.do_not_log)
    }

    pub fn should_index(&self, page_content: &PageContent) -> bool {
        !DocumentMatcher::matches_any(page_content, &self.do_not_index)
            && !DocumentMatcher::matches_any(page_content, &self.do_not_log)
    }

    /// Should we display information in clear about this page in the search
    /// results.
    pub fn should_display(&self, page_content: &PageContent) -> bool {
        !DocumentMatcher::matches_any(page_content, &self.hidden)
    }

    pub fn fetch(connection: &Connection) -> WResult<Restrictions> {
        let all = url_restrictions::fetch_all(connection)?;
        Self::build(&all)
    }

    pub(crate) fn build(all: &[UrlRestriction]) -> WResult<Restrictions> {
        let mut do_not_log = Vec::new();
        let mut do_not_index = Vec::new();
        let mut do_index = Vec::new();
        let mut hidden = Vec::new();
        for one in all {
            let policy = one.kind.parse::<StorePolicy>()?;
            match policy {
                StorePolicy::NoLog => do_not_log.push(one.into()),
                StorePolicy::NoIndex => do_not_index.push(one.into()),
                StorePolicy::DoIndex => do_index.push(one.into()),
                StorePolicy::Hidden => hidden.push(one.into()),
            }
        }
        Ok(Restrictions {
            do_not_log,
            do_not_index,
            do_index,
            hidden,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Build a test PageContent with only the URL.
    fn pc(url: &str) -> PageContent {
        PageContent {
            url: url.into(),
            ..PageContent::default()
        }
    }

    fn re(s: &str) -> Regex {
        Regex::new(s).unwrap()
    }

    #[test]
    fn test_matches_restriction() {
        assert_eq!(matches_restriction("https://foo", &re("https://foo")), true);
        assert_eq!(
            matches_restriction("https://foo", &re("https://baz")),
            false
        );
    }

    #[test]
    fn test_not_index() {
        let restrictions = Restrictions {
            do_not_index: vec![DocumentMatcher::with_url("https://foo").unwrap()],
            ..Restrictions::default()
        };
        assert_eq!(restrictions.should_index(&pc("https://foo")), false);
        assert_eq!(restrictions.should_index(&pc("https://baz")), true);
    }

    #[test]
    fn test_not_log_propagates_to_not_index() {
        let restrictions = Restrictions {
            do_not_log: vec![DocumentMatcher::with_url("https://foo").unwrap()],
            ..Restrictions::default()
        };
        assert_eq!(restrictions.should_index(&pc("https://foo")), false);
        assert_eq!(restrictions.should_index(&pc("https://baz")), true);
    }

    #[test]
    fn test_hide_with_title() {
        let mut dm = DocumentMatcher::with_url("https://foo").unwrap();
        dm.title = Some(re("title"));
        let restrictions = Restrictions {
            hidden: vec![dm],
            ..Restrictions::default()
        };

        // Should match when both url and title are provided.
        let pc = PageContent {
            url: "https://foo".into(),
            title: "Some title".into(),
            ..PageContent::default()
        };
        assert_eq!(restrictions.should_display(&pc), false);

        // Should match when both url and title are provided.
        let pc = PageContent {
            url: "https://foo".into(),
            title: "Some other".into(),
            ..PageContent::default()
        };
        assert_eq!(restrictions.should_display(&pc), true);
    }

    #[test]
    fn test_build() {
        let all = vec![UrlRestriction::with_url(
            &StorePolicy::Hidden,
            "https://foo",
        )];

        let restrictions = Restrictions::build(&all).unwrap();

        // not matching
        let display_one = restrictions.should_display(&pc("https://one"));
        assert!(display_one);

        // matching
        let display_foo = restrictions.should_display(&pc("https://foo"));
        assert!(!display_foo);
    }
}
