//! Representes a denormalized action which can be used in the UI.
//!
use config;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FormattedAction {
    pub annotation: Option<String>,
    pub id: usize,
    pub epic: Option<String>,
    pub kind: String,
    pub name: String,
    pub location: Option<String>,
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
