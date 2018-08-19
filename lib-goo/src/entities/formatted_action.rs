//! Representes a denormalized action which can be used in the UI.
//!
use date::Date;

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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FormattedAction {
    pub annotation: Option<String>,
    pub id: usize,
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
    pub sequence: Vec<usize>,
    // Starting points where this sequence repeats.
    pub anchors: Vec<usize>,
}
