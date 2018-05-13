use ::config;


#[derive(Clone, Debug, Default)]
pub struct FormattedAction {
    pub annotation: Option<String>,
    pub id: usize,
    pub epic: Option<String>,
    pub kind: String,
    pub name: String,
    pub location: Option<String>,
}


impl FormattedAction {
    pub fn to_shell_command(self, content: &config::Content, env: &config::Environment) -> String {
        use config::Content::*;

        match *content {
            Path => {
                self.location.map(|a| format!("cd {}", env.localized_path(&a)))
                    .unwrap_or(String::from(""))
            }
            PathWithCommand => {
                if self.location.is_some() {
                    format!("cd {} && {}", env.localized_path(&self.location.unwrap()), self.name)
                } else {
                    self.name
                }
            }
            Command => self.name
        }
    }


}

