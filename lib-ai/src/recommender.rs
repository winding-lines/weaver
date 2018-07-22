/// Build a list of recommended actions from the historical list of actions and current context.
use lib_goo::entities::{FormattedAction, RecommendReason};

fn to_recommended(action: &FormattedAction) -> FormattedAction {
    let mut out = action.clone();
    out.id = 0;
    out.reason = RecommendReason::CorrelatedWithCommand;
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
        out.push(to_recommended(recent_1));
        return out;
    }
    // find earlier instances of the current actions and recommend the one following it,
    // since this is a reverse iterator we need the earlier one.
    let mut earlier: Option<&FormattedAction> = None;
    let name = &recent_1.name;
    for other in history.iter().rev().skip(1) {
        if &other.name == name {
            if earlier.is_some() {
                break;
            }
        } else {
            earlier = Some(other);
        }
    }
    if let Some(earlier) = earlier {
        out.push(to_recommended(earlier))
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
        let history: Vec<FormattedAction> = vec!["foo", "bar", "foo", "foo", "foo"]
            .into_iter()
            .map(|a| FormattedAction {
                name: a.into(),
                ..FormattedAction::default()
            })
            .collect();
        let r = recommend(&history);
        assert_eq!(&r.first().unwrap().name, "bar");
    }
}
