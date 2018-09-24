use super::epic::Epic;
use lib_error::*;
use serde_json as json;

/// All the Milestones that Weaver knows about.
#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct Weaver {
    pub active_epic: Option<String>,
    // Name of the active milestone.
    pub active_flow: Option<String>,
    // Active flow being executed.
    milestones: Vec<Epic>,
    // Start in server mode if it doesn't run.
    pub start_server: Option<bool>,
}

impl Default for Weaver {
    fn default() -> Weaver {
        Weaver {
            active_epic: None,
            active_flow: None,
            milestones: Vec::new(),
            start_server: Some(true),
        }
    }
}

impl Weaver {
    pub fn load_from_string(contents: &str) -> Result<Weaver> {
        let weaver: Weaver = json::from_str(&contents).context("parsing main weaver state".into())?;
        Ok(weaver)
    }

    pub fn to_str(&self) -> Result<String> {
        let pretty = json::to_string_pretty(self).context("encoding weaver in json".into())?;
        Ok(pretty)
    }
}
