/// Build a list of recommended actions from the historical list of actions and current context.
use lib_goo::entities::{FormattedAction, RecommendReason};
use std::collections::HashMap;

use chrono::prelude::*;

/// Build a recommended action based on most recent.
fn to_recommended_recent(action: &FormattedAction) -> FormattedAction {
    let mut out = action.clone();
    let age = if let Ok(when) = DateTime::parse_from_rfc3339("") {
        Utc::now().signed_duration_since(when).num_minutes()
    } else {
        0
    };
    out.id = 0;
    out.reason = RecommendReason::CorrelatedMostRecent(age);
    out
}

/// Build a recommended action based on frequency.
fn to_recommended_frequent(action: &FormattedAction, name: String, repeat: u32) -> FormattedAction {
    let mut out = action.clone();
    out.name = name;
    out.id = 0;
    out.reason = RecommendReason::CorrelatedMostFrequent(repeat);
    out
}

// The following will be returned:
// - Previous folder unless there is just one command in the current folder,
//   in that case return the folder before.
// - If first command in folder find the first command that was executed last
//   time right after command change.
// - Find the earlier instances of the last command and return the most recent
//   and most frequent commands.

pub fn recommend(history: &[FormattedAction]) -> Vec<FormattedAction> {
    let mut out = Vec::new();
    if history.is_empty() {
        return out;
    }
    let recent_1 = history.last().unwrap();
    if history.len() == 1 {
        out.push(to_recommended_recent(recent_1));
        return out;
    }
    // find earlier instances of the current actions and recommend the one following it,
    let name = &recent_1.name;
    let mut earlier = None;

    // Track the most frequently used earliest command.
    let mut counts = HashMap::new();
    let mut most_frequent: Option<(String, usize)> = None;

    let first_iter = history.iter().rev().skip(1);
    let second_iter = history.iter().rev().skip(2);
    for (first, second) in first_iter.zip(second_iter) {
        if &second.name == name && &first.name != name {
            if earlier.is_none() {
                earlier = Some(first);
            }
            if first.name != earlier.unwrap().name {
                let entry = counts.entry(&first.name).or_insert(0);
                *entry += 1;
                let previous_max = most_frequent.as_ref().map(|a| a.1).unwrap_or(0);
                if *entry > previous_max {
                    most_frequent = Some((first.name.clone(), *entry));
                };
            }
        }
    }
    if let Some(earlier) = earlier {
        debug!("Adding earlier {:?}", earlier);
        out.push(to_recommended_recent(earlier))
    }
    if let Some(mf) = most_frequent {
        debug!("Adding more frequent {:?}", mf);
        let mut freq = to_recommended_frequent(recent_1, mf.0, mf.1 as u32);
        out.push(freq);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_recommendations_for_empty_history() {
        let r = recommend(&Vec::new());
        assert!(r.is_empty());
    }

    #[test]
    fn recommend_from_history_with_single_entry() {
        let history = vec![FormattedAction {
            name: "foo".into(),
            ..FormattedAction::default()
        }];
        let r = recommend(&history);
        assert_eq!(&r.first().unwrap().name, "foo");
    }

    #[test]
    fn recommend_from_earlier() {
        let history: Vec<FormattedAction> = vec!["baz", "foo", "bar", "foo", "foo", "foo"]
            .into_iter()
            .map(|a| FormattedAction {
                name: a.into(),
                ..FormattedAction::default()
            })
            .collect();
        let r = recommend(&history);
        assert_eq!(r.len(), 1);
        assert_eq!(&r.first().unwrap().name, "bar");
    }
    #[test]
    fn recommend_from_earlier_and_frequent() {
        let history: Vec<FormattedAction> = vec![
            "foo", "baz", "foo", "baz", "foo", "bar", "foo", "foo", "foo",
        ].into_iter()
            .map(|a| FormattedAction {
                name: a.into(),
                ..FormattedAction::default()
            })
            .collect();
        let r = recommend(&history);
        assert_eq!(r.len(), 2);
        assert_eq!(&r.first().unwrap().name, "bar");
        assert_eq!(&r[1].name, "baz");
    }
}
