//! Representes a denormalized action which can be used in the UI.
//!
use config;

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
}

impl FormattedAction {
    pub fn into_shell_command(
        self,
        content: &config::Content,
        _env: &config::Environment,
    ) -> String {
        use config::Content::*;

        match *content {
            Path => self.location
                .map(|a| format!("cd {}", a))
                .unwrap_or_else(String::new),
            PathWithCommand => match self.location {
                Some(a) => format!("cd {} && {}", a, self.name),
                None => self.name,
            },
            Command => self.name,
        }
    }
}
