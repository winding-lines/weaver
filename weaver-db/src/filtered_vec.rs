use entities::FormattedAction;

/// Provide a filtered view on top of a vector of FormattedActions.
/// Will generalize as needed.
pub struct FilteredVec {
    content: Vec<FormattedAction>,
    rows: usize,
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

    pub fn find_previous(&self, search: &str, current: usize)  -> Option<usize> {
        let size = self.content.len();

        for i in 1..size {
            // look through all the array, wrap around at the end
            let pos = if current >= i {
                current - i
            } else {
                current + size - i
            };

            if let Some(action) = self.content.get(pos) {
                if action.name.contains(search) {
                    return Some(pos);
                }
            }
        }
        None
    }

    pub fn find_next(&self, search: &str, current: usize)  -> Option<usize> {
        let size = self.content.len();

        for i in 1..size {
            // look through all the array, wrap around at the end
            let pos = if current + i < size  {
                current + i
            } else {
                current + i - size
            };

            if let Some(action) = self.content.get(pos) {
                if action.name.contains(search) {
                    return Some(pos);
                }
            }
        }
        None
    }

    /// Build a vector with the entries that match this filter.
    /// For None returns a new vector.
    pub fn filter(&mut self, filter: Option<&str>) -> Vec<FormattedAction> {
        let mut content: Vec<FormattedAction> = Vec::new();


        if let Some(f) = filter {
            for entry in &self.content {
                if entry.name.contains(f)
                    || entry.epic.as_ref().map(|e| e.contains(f)).unwrap_or(false)
                    || f.is_empty() {
                    content.push(entry.clone());
                }
            }
        } else {
            content.extend_from_slice(&self.content);
        }

        // Make sure content is at the bottom of the screen
        while content.len() < self.rows {
            content.insert(0, FormattedAction::default() )
        };

        content
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prev() {
        let table = build_table();

        assert_eq!(Some(0), table.find_previous("abc",1));
        assert_eq!(None, table.find_previous("def", 1));
        assert_eq!(Some(1), table.find_previous("abc",0));
    }

    #[test]
    fn test_next() {
        let table = build_table();

        assert_eq!(Some(1), table.find_next("abc",0));
        assert_eq!(None, table.find_next("def", 1));
        // wrap around
        assert_eq!(Some(0), table.find_next("abc",1));
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
