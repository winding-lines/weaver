use ::errors::*;
use serde_json as json;
use super::Milestone;

/// All the Milestones that Weaver knows about.
#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Weaver {
    pub active_milestone: Option<String>, // Name of the active milestone.
    pub active_flow: Option<String>, // Active flow being executed.
    milestones: Vec<Milestone>
}

impl Weaver {
    pub fn load_from_string(contents: &str) -> Result<Weaver> {
        let weaver: Weaver = json::from_str(&contents)
        .chain_err(|| "parsing main weaver state")?;
        Ok(weaver)
    }

    pub fn to_str(&self) -> Result<String> {
        json::to_string_pretty(self).chain_err(|| "encoding weaver in json")
    }
}

