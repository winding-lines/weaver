//! Representes a denormalized action which can be used in the UI.
//!
use date::Date;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Reasons why an action may be recommended.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum RecommendReason {
    /// This command occured in the history of commands in the order in which it is returned.
    Historical,
    /// Has happened frequently after the command which has just executed.
    CorrelatedMostRecent(i64),
    /// Has occured the most times after the command which has just completed.
    CorrelatedMostFrequent(u32),
    /// The user has selected or typed this command during the current run.
    UserSelected,
}
impl Default for RecommendReason {
    fn default() -> Self {
        RecommendReason::Historical
    }
}

// Create a new type for the Action ID field, there are a lot of functions
// handling both IDs and raw indices in arrays.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize, Eq, Hash)]
pub struct ActionId(usize);

impl ActionId {
    pub fn new(u: usize) -> ActionId {
        ActionId(u)
    }
    pub fn format(&self) -> Option<String> {
        if self.0 == 0 {
            None
        } else {
            Some(format!("{}", self.0))
        }
    }

    pub fn next(&self) -> usize {
        self.0 + 1
    }

    pub fn prev(&self) -> usize {
        self.0 - 1
    }

    pub fn is_before(&self, other: &ActionId) -> bool {
        self.0 < other.0
    }
}

impl Display for ActionId {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FormattedAction {
    pub annotation: Option<String>,
    pub id: ActionId,
    pub epic: Option<String>,
    pub kind: String,
    pub name: String,
    pub location: Option<String>,
    /// The reason why this action is being recommended.
    pub reason: RecommendReason,
    pub when: Option<Date>,
}

impl FormattedAction {
    pub fn into_shell_command(self) -> String {
        self.name
    }
}

// Structure to represent repeats in the list of actions.
#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Cycle {
    // Synthetic ID for this cyle, not persistent.
    pub id: usize,
    // Sequence of the most recent IDs that repeat. Note that one action id
    // can belong to multiple cycles.
    pub sequence: Vec<ActionId>,
    // Starting points where this sequence repeats.
    pub anchors: Vec<ActionId>,
}

impl Cycle {
    // Check to see if current and other cycles have overlapping sequences.
    pub fn overlaps(&self, other: &Cycle) -> bool {
        let first1 = match self.sequence.get(0) {
            Some(id) => id,
            None => return false,
        };
        let first2 = match other.sequence.get(0) {
            Some(id) => id,
            None => return false,
        };
        if first1.is_before(first2) {
            self.sequence.contains(first2)
        } else {
            other.sequence.contains(first1)
        }
    }

    // Remove successive cycles when they are overlapping. The processing is
    // done in place.
    pub fn remove_overlapping(cycles: &mut Vec<Cycle>) {
        // If there are overlapping sets only keep the very first one, the extra
        // sets do not provide value.
        let mut index_to_remove = Vec::new();
        for (pos, (one, two)) in cycles.iter().zip(cycles.iter().skip(1)).enumerate() {
            if one.overlaps(two) {
                index_to_remove.push(pos + 1);
            }
        }
        for r in index_to_remove.iter().rev() {
            cycles.remove(*r);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlap() {
        let one = Cycle {
            id: 1,
            anchors: vec![ActionId::new(0)],
            sequence: vec![ActionId::new(1), ActionId::new(2)],
        };
        let two = Cycle {
            id: 2,
            anchors: vec![ActionId::new(1)],
            sequence: vec![ActionId::new(2), ActionId::new(3)],
        };
        assert!(one.overlaps(&two));
        assert!(two.overlaps(&one));
    }

    #[test]
    fn test_non_overlap() {
        let one = Cycle {
            id: 1,
            anchors: vec![ActionId::new(0)],
            sequence: vec![ActionId::new(1), ActionId::new(2)],
        };
        let two = Cycle {
            id: 2,
            anchors: vec![ActionId::new(1)],
            sequence: vec![ActionId::new(3), ActionId::new(4)],
        };
        assert!(!one.overlaps(&two));
        assert!(!two.overlaps(&one));
    }

    #[test]
    fn test_remove() {
        let one = Cycle {
            id: 1,
            anchors: vec![ActionId::new(0)],
            sequence: vec![ActionId::new(1), ActionId::new(2)],
        };
        let two = Cycle {
            id: 2,
            anchors: vec![ActionId::new(1)],
            sequence: vec![ActionId::new(2), ActionId::new(3)],
        };
        let mut data = vec![one, two];
        Cycle::remove_overlapping(&mut data);
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].id, 1);
    }
}
