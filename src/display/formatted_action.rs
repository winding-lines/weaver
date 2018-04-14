use ::config;


#[derive(Clone, Debug)]
pub struct FormattedAction {
    pub annotation: Option<String>,
    pub id: usize,
    pub epic: Option<String>,
    pub kind: String,
    pub name: String,
    pub location: Option<String>,
}

impl FormattedAction {
    pub fn as_shell_command(self, env: &config::Content) -> String {
        use config::Content::*;

        match *env {
            Path => {
                self.location.map(|a| format!("cd {}", a))
                    .unwrap_or(String::from(""))
            }
            PathWithCommand => {
                if self.location.is_some() {
                    format!("cd {} && {}", self.location.unwrap(), self.name)
                } else {
                    self.name
                }
            }
            Command => self.name
        }
    }


}

