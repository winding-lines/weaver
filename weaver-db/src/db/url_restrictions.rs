use ::backends::schema::url_restrictions;
use ::Connection;
use diesel;
use diesel::prelude::*;
use weaver_error::{Result, ResultExt};

const NO_LOG: &str = "!log";
const NO_INDEX: &str = "!ndx";

pub struct Restrictions {
    pub do_not_log: Vec<String>,
    pub do_not_index: Vec<String>,
}

fn matches_restriction(url: &str, restriction: &str) -> bool {
    if url == restriction {
        return true;
    };
    false
}

fn matches_any_restriction(url: &str, restrictions: &[String]) -> bool {
    for restriction in restrictions {
        if matches_restriction(url, &restriction) {
            return true;
        }
    }
    false
}

impl Restrictions {
    pub fn should_index(&self, url: &str) -> bool {
        !matches_any_restriction(url, &self.do_not_index)
            &&!matches_any_restriction(url, &self.do_not_log)
    }
}


pub fn insert(connection: &Connection, kind: &str, url_expr: &str) -> Result<()> {
    if kind != NO_INDEX && kind != NO_LOG {
        return Err("bad kind parameter for url_restriction".into());
    }
    diesel::insert_into(url_restrictions::table)
        .values((url_restrictions::dsl::kind.eq(kind),
                 url_restrictions::dsl::url_expr.eq(url_expr)))
        .execute(connection)
        .chain_err(|| "insert into url_restrictions")?;
    Ok(())
}

pub fn fetch_all(connection: &Connection) -> Result<Restrictions> {
    let all = url_restrictions::dsl::url_restrictions
        .select((url_restrictions::dsl::kind, url_restrictions::dsl::url_expr))
        .load::<(String, String)>(connection)
        .chain_err(|| "fetch all")?;
    let mut do_not_log = Vec::new();
    let mut do_not_index = Vec::new();
    for (kind, url_exp) in all {
        let mut c = kind.chars();
        match (c.next(), c.next()) {
            (Some('!'), Some('l')) => do_not_log.push(url_exp),
            (Some('!'), Some('n')) => do_not_index.push(url_exp),
            _ => return Err(format!("unknown url restriction kind {}", kind).into())
        }
    }
    Ok(Restrictions {
        do_not_log,
        do_not_index,
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
            do_not_log: vec![]
        };
        assert_eq!(restrictions.should_index("https://foo"), false);
        assert_eq!(restrictions.should_index("https://baz"), true);
    }

    #[test]
    fn test_not_log_propagates_to_not_index() {
        let restrictions = Restrictions {
            do_not_log: vec!["https://foo".into()],
            do_not_index: vec![]
        };
        assert_eq!(restrictions.should_index("https://foo"), false);
        assert_eq!(restrictions.should_index("https://baz"), true);
    }
}
