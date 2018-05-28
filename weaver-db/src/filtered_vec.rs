use entities::FormattedAction;
use regex::Regex;

/// Provide a filtered view on top of a vector of FormattedActions.
/// Will generalize as needed.
pub struct FilteredVec {
    content: Vec<FormattedAction>,
    rows: usize,
}

fn str_to_regex(s: &str) -> Regex {
    Regex::new(s).expect("valid regex")
}

struct Matcher {
    r: Regex,
}

impl Matcher {
    pub fn build(s: &str) -> Matcher {
        Matcher {
            r: str_to_regex(s),
        }
    }

    fn is_match(&self, fa: &FormattedAction) -> bool {
        self.r.is_match(&fa.name)
    }
}


impl FilteredVec {
    pub fn new(content: Vec<FormattedAction>, rows: usize) -> FilteredVec {
        FilteredVec {
            content,
            rows,
        }
    }

    pub fn get(&self, i: usize) -> Option<FormattedAction> {
        self.content.get(i).cloned()
    }

    pub fn find_previous(&self, search: &str, current: usize) -> Option<usize> {
        let size = self.content.len();
        let matcher = Matcher::build(search);

        for i in 1..size {
            // look through all the array, wrap around at the end
            let pos = if current >= i {
                current - i
            } else {
                current + size - i
            };

            if let Some(action) = self.content.get(pos) {
                if matcher.is_match(action) {
                    return Some(pos);
                }
            }
        }
        None
    }

    pub fn find_next(&self, search: &str, current: usize) -> Option<usize> {
        let size = self.content.len();
        let matcher = Matcher::build(search);

        for i in 1..size {
            // look through all the array, wrap around at the end
            let pos = if current + i < size {
                current + i
            } else {
                current + i - size
            };

            if let Some(action) = self.content.get(pos) {
                if matcher.is_match(action) {
                    return Some(pos);
                }
            }
        }
        None
    }

    /// Build a vector with the entries that match this filter.
    /// For None returns a new vector.
    pub fn filter(&self, filter: Option<&str>) -> Vec<FormattedAction> {
        let mut content: Vec<FormattedAction> = Vec::new();


        if let Some(f) = filter {
            let matcher = Matcher::build(f);
            for entry in &self.content {
                if matcher.is_match(entry) {
                    content.push(entry.clone());
                }
            }
        } else {
            content.extend_from_slice(&self.content);
        }

        // Make sure content is at the bottom of the screen
        while content.len() < self.rows {
            content.insert(0, FormattedAction::default())
        };

        content
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filtered_prev() {
        let table = build_table();

        assert_eq!(Some(0), table.find_previous("abc", 1));
        assert_eq!(None, table.find_previous("def", 1));
        assert_eq!(Some(1), table.find_previous("abc", 0));
    }

    #[test]
    fn test_filtered_next() {
        let table = build_table();

        assert_eq!(Some(1), table.find_next("abc", 0));
        assert_eq!(None, table.find_next("def", 1));
        // wrap around
        assert_eq!(Some(0), table.find_next("abc", 1));
    }

    #[test]
    fn test_filtered_filter_no_matches() {
        let table = build_table();
        let filtered = table.filter(Some("z"));

        assert_eq!(2, filtered.len(), "always have the required number of entries, even if no matches");
        assert_eq!("", filtered[0].name, "when no matches the name is empty");
    }

    #[test]
    fn test_match_simple_string() {
        let abc = FormattedAction {
            name: "abc".into(),
            id: 0,
            ..Default::default()
        };
        let def = FormattedAction {
            name: "def".into(),
            id: 1,
            ..Default::default()
        };
        let matcher = Matcher::build("e");
        assert_eq!(true, matcher.is_match(&def), "'e' matches 'def'");
        assert_eq!(false, matcher.is_match(&abc), "'e' does not match 'abc'");
    }

    #[test]
    fn test_match_regex() {
        let abc = FormattedAction {
            name: "abc".into(),
            id: 0,
            ..Default::default()
        };
        let matcher = Matcher::build("^b");
        assert_eq!(false, matcher.is_match(&abc), "'^b' does not match 'abc'");
        let matcher = Matcher::build("c$");
        assert_eq!(true, matcher.is_match(&abc), "'c$' matches 'abc'");
        let matcher = Matcher::build("a.*c");
        assert_eq!(true, matcher.is_match(&abc), "'a.*c' matches 'abc'");
    }

    fn build_table() -> FilteredVec {
        FilteredVec::new(vec![
            FormattedAction {
                name: "abc".into(),
                id: 0,
                ..Default::default()
            },
            FormattedAction {
                name: "abcde".into(),
                id: 1,
                ..Default::default()
            },
        ], 2)
    }
}
