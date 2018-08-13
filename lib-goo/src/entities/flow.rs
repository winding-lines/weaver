//! Experimental entity to represent a sequence of actions.
//!
use lib_error::*;
use serde_json as json;
use std::fs::File;
use std::io::prelude::*;

/// To be implemented by Preconditions.
trait Matcher {
    fn matches(&self) -> bool;
}

/// Precondition that matches if a certain file contains a string.
#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct FileContent {
    path: String,
    contains: String,
}

impl Matcher for FileContent {
    fn matches(&self) -> bool {
        if let Ok(mut file) = File::open(&self.path) {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                return contents.contains(self.contains.as_str());
            }
        }
        false
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub enum Precondition {
    #[serde(rename = "file_content")]
    FileContent(FileContent),
}

/// Pass through the Matcher to all the variants.
impl Matcher for Precondition {
    fn matches(&self) -> bool {
        match self {
            Precondition::FileContent(ref fc) => fc.matches(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Flow {
    pub name: String,
    pub preconditions: Vec<Precondition>,
    pub actions: Vec<String>,
}

impl Flow {
    pub fn load_from_string(contents: &str) -> Result<Flow> {
        let script: Flow = json::from_str(&contents).chain_err(|| "parsing flow")?;
        Ok(script)
    }

    pub fn to_str(&self) -> Result<String> {
        json::to_string_pretty(self).chain_err(|| "encoding script in json")
    }

    /// Check if the preconditions of the Flow match.
    pub fn matches(&self) -> bool {
        for pre in &self.preconditions {
            if !pre.matches() {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_parses() {
        use super::*;

        let data = r#"
        {
            "name": "foo",
            "preconditions": [
                {"file_content": {
                    "path": ".env",
                    "contains": "DATABASE_URL"
                }}
            ],
            "actions": [
                "diesel setup"
            ]
        }
        "#;
        let res = Flow::load_from_string(data);
        assert!(res.is_ok(), format!("unexpected error {:?}", res));
        let flow = res.unwrap();
        assert_eq!(flow.name, "foo");
        let expected_pre = vec![Precondition::FileContent(FileContent {
            path: String::from(".env"),
            contains: String::from("DATABASE_URL"),
        })];
        assert_eq!(flow.preconditions.as_slice(), expected_pre.as_slice());
        assert_eq!(
            flow.actions.as_slice(),
            vec![String::from("diesel setup")].as_slice()
        );
    }
}
