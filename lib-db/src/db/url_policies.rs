use ::backends::schema::url_restrictions;
use ::Connection;
use diesel;
use diesel::prelude::*;
use std::str::FromStr;
use std::string::ToString;
use lib_error::{Result as WResult, ResultExt};

// Url (patterns) to not log, by default all urls are logged.
pub const NO_LOG: &str = "!log";

// Url (patterns) to index, by default no urls get indexed so this is a white list.
pub const DO_INDEX: &str = "ndx";

// Url (patterns) to not index.
pub const NO_INDEX: &str = "!ndx";

pub enum UrlPolicy {
    NoLog,
    NoIndex,
    DoIndex,
}

impl FromStr for UrlPolicy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        match s {
            NO_LOG => Ok(UrlPolicy::NoLog),
            DO_INDEX => Ok(UrlPolicy::DoIndex),
            NO_INDEX => Ok(UrlPolicy::NoIndex),
            _ => Err(format!("cannot parse url policy from {}", s))
        }
    }
}

impl ToString for UrlPolicy {
    fn to_string(&self) -> String {
        match *self {
            UrlPolicy::NoLog => NO_LOG.into(),
            UrlPolicy::NoIndex => NO_INDEX.into(),
            UrlPolicy::DoIndex => DO_INDEX.into(),
        }
    }
}

impl UrlPolicy {
    // Return the DB representation for the enum.
    #[inline]
    fn to_db(&self) -> &'static str {
        match *self {
            UrlPolicy::NoLog => NO_LOG,
            UrlPolicy::NoIndex => NO_INDEX,
            UrlPolicy::DoIndex => DO_INDEX,
        }
    }
}

// Structure used to manipulate in memory all the restrictions and also to communicate
// them with the client.
#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Restrictions {
    pub do_not_log: Vec<String>,
    pub do_not_index: Vec<String>,
    // Note: this field is used directly on the client (chrome extension) since the user can
    // force index any page.
    pub do_index: Vec<String>,
}

// Check if the incoming URL matches against the pre-defined restriction.
fn matches_restriction(url: &str, restriction: &str) -> bool {
    if url == restriction {
        return true;
    };
    false
}

// Check if the incoming URL matches against any of the pre-defined restrictions
fn matches_any_restriction(url: &str, restrictions: &[String]) -> bool {
    for restriction in restrictions {
        if matches_restriction(url, &restriction) {
            return true;
        }
    }
    false
}

impl Restrictions {
    // Check if there are any restrictions against indexing this URL.
    // Note that from the UI the user can force the index of any page
    // so we do not check against the do_index field.
    pub fn should_index(&self, url: &str) -> bool {
        !matches_any_restriction(url, &self.do_not_index)
            && !matches_any_restriction(url, &self.do_not_log)
    }
}

// Fetch the id of the entry, if it already exists
fn fetch_id(connection: &Connection, policy: &UrlPolicy, url_expr: &str) -> WResult<Option<i32>> {
    let existing = url_restrictions::dsl::url_restrictions
        .filter(url_restrictions::dsl::url_expr.eq(&url_expr))
        .filter(url_restrictions::dsl::kind.eq(policy.to_db()))
        .select(url_restrictions::dsl::id)
        .load::<Option<i32>>(connection)
        .chain_err(|| "testing existence in epics table")?;
    Ok(existing.iter().next().map(|a| a.expect("must have id")))
}


// Insert a new restriction, unless it is already present.
pub fn insert(connection: &Connection, kind: &UrlPolicy, url_expr: &str) -> WResult<()> {
    let existing = fetch_id(connection, kind, url_expr)?;
    if existing.is_none() {
        diesel::insert_into(url_restrictions::table)
            .values((url_restrictions::dsl::kind.eq(kind.to_db()),
                     url_restrictions::dsl::url_expr.eq(url_expr)))
            .execute(connection)
            .chain_err(|| "insert into url_restrictions")?;
    }
    Ok(())
}

pub fn fetch_all(connection: &Connection) -> WResult<Restrictions> {
    let all = url_restrictions::dsl::url_restrictions
        .select((url_restrictions::dsl::kind, url_restrictions::dsl::url_expr))
        .load::<(String, String)>(connection)
        .chain_err(|| "fetch all")?;
    let mut do_not_log = Vec::new();
    let mut do_not_index = Vec::new();
    let mut do_index = Vec::new();
    for (kind, url_exp) in all {
        let policy = kind.parse::<UrlPolicy>()?;
        match policy {
            UrlPolicy::NoLog => do_not_log.push(url_exp),
            UrlPolicy::NoIndex => do_not_index.push(url_exp),
            UrlPolicy::DoIndex => do_index.push(url_exp),
        }
    }
    Ok(Restrictions {
        do_not_log,
        do_not_index,
        do_index,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_restriction() {
        assert_eq!(matches_restriction("https://foo", "https://foo"), true);
        assert_eq!(matches_restriction("https://foo", "https://baz"), false);
    }

    #[test]
    fn test_not_index() {
        let restrictions = Restrictions {
            do_not_index: vec!["https://foo".into()],
            ..Restrictions::default()
        };
        assert_eq!(restrictions.should_index("https://foo"), false);
        assert_eq!(restrictions.should_index("https://baz"), true);
    }

    #[test]
    fn test_not_log_propagates_to_not_index() {
        let restrictions = Restrictions {
            do_not_log: vec!["https://foo".into()],
            ..Restrictions::default()
        };
        assert_eq!(restrictions.should_index("https://foo"), false);
        assert_eq!(restrictions.should_index("https://baz"), true);
    }
}
