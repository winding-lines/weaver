use lib_error::*;
/// Provide a filtered view on top of a vector of items.
use regex::Regex;

pub trait FilteredItem {
    fn is_match(&self, regex: &Regex) -> bool;
}
pub struct FilteredVec<T> {
    content: Vec<T>,
    rows: usize,
}

struct Matcher(Regex);

impl Matcher {
    pub fn build(s: &str) -> Result<Matcher> {
        let r = Regex::new(s).chain_err(|| "invalid regex")?;
        Ok(Matcher(r))
    }

    fn is_match<F: FilteredItem>(&self, fa: &F) -> bool {
        fa.is_match(&self.0)
    }
}

impl<T: FilteredItem + Clone + Default> FilteredVec<T> {
    pub fn new(content: Vec<T>, rows: usize) -> FilteredVec<T> {
        FilteredVec { content, rows }
    }

    pub fn get(&self, i: usize) -> Option<T> {
        self.content.get(i).cloned()
    }

    pub fn find_previous(&self, search: &str, current: usize) -> Option<usize> {
        let size = self.content.len();
        let matcher = match Matcher::build(search) {
            Ok(r) => r,
            Err(e) => {
                error!("bad matcher: {:?}", e);
                return None;
            }
        };

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
        let matcher = match Matcher::build(search) {
            Ok(r) => r,
            Err(e) => {
                error!("bad matcher: {:?}", e);
                return None;
            }
        };

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
    pub fn filter(&self, filter: Option<&str>) -> Vec<T> {
        let mut content: Vec<T> = Vec::new();

        // Filter if a search item is passed in.
        if let Some(search) = filter {
            match Matcher::build(search) {
                // Only filter if the matcher is valid.
                Ok(matcher) => {
                    for entry in &self.content {
                        if matcher.is_match(entry) {
                            content.push(entry.clone());
                        }
                    }
                }
                Err(e) => {
                    // TODO: log the error in a notification in the UI.
                    error!("bad matcher: {:?}", e);
                }
            };
        } else {
            content.extend_from_slice(&self.content);
        }

        // Make sure content is at the bottom of the screen
        while content.len() < self.rows {
            content.insert(0, T::default())
        }

        content
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Default)]
    struct Name(String);

    impl FilteredItem for Name {
        fn is_match(&self, regex: &Regex) -> bool {
            regex.is_match(&self.0)
        }
    }

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

        assert_eq!(
            2,
            filtered.len(),
            "always have the required number of entries, even if no matches"
        );
        assert_eq!("", filtered[0].0, "when no matches the name is empty");
    }

    #[test]
    fn test_match_simple_string() {
        let abc = Name("abc".into());
        let def = Name("def".into());
        let matcher = Matcher::build("e").unwrap();
        assert_eq!(true, matcher.is_match(&def), "'e' matches 'def'");
        assert_eq!(false, matcher.is_match(&abc), "'e' does not match 'abc'");
    }

    #[test]
    fn test_match_regex() {
        let abc = Name("abc".into());
        let matcher = Matcher::build("^b").unwrap();
        assert_eq!(false, matcher.is_match(&abc), "'^b' does not match 'abc'");
        let matcher = Matcher::build("c$").unwrap();
        assert_eq!(true, matcher.is_match(&abc), "'c$' matches 'abc'");
        let matcher = Matcher::build("a.*c").unwrap();
        assert_eq!(true, matcher.is_match(&abc), "'a.*c' matches 'abc'");
    }

    fn build_table() -> FilteredVec<Name> {
        FilteredVec::new(vec![Name("abc".into()), Name("abcde".into())], 2)
    }
}
