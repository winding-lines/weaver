use lib_goo::entities::{Cycle, FormattedAction};
use std::collections::HashSet;

fn save_cycle(existing: &mut Vec<Cycle>, dups: &mut Vec<usize>) {
    if !dups.is_empty() {
        // Create a cycle:
        // - the sequence is just one instruction since this is what is repeatings
        // - the anchors are all the spots in which the instruction appears
        let mut anchors: Vec<usize> = dups.drain(..).collect();
        // Now reverse the anchors as per the API.
        anchors.reverse();
        // Get the very last id.
        let sequence = vec![anchors[anchors.len() - 1]];
        // Assign a synthetic ID to the cycle.
        let id = existing.len();
        existing.push(Cycle {
            id,
            anchors,
            sequence,
        });
    }
}

// Extract cycles from the list of actions. The repeating sequences are
// extracted from the input array.
pub fn extract_cycles(actions: &Vec<FormattedAction>) -> Vec<Cycle> {
    let mut out = Vec::new();
    let mut it = actions.iter().rev();
    let mut earlier = match it.next() {
        Some(action) => action,
        None => return out,
    };
    let mut dups = Vec::new();
    for action in it {
        if action.name == earlier.name {
            if dups.is_empty() {
                dups.push(earlier.id);
            };
            dups.push(action.id);
        } else {
            earlier = action;
            save_cycle(&mut out, &mut dups);
        }
    }
    save_cycle(&mut out, &mut dups);
    out
}

// Remove the duplicate cycles from the formatted actions.
pub fn decycle(actions: &mut Vec<FormattedAction>, cycles: &[Cycle]) {
    // Set of must-keep actions: they are part of one of the sequences that
    // define a cycle.
    let mut must_keep = HashSet::new();

    // For a cycle we can delete all the repeats starting at the anchors.
    let mut could_delete = HashSet::new();

    for cycle in cycles.iter() {
        for is in &cycle.sequence {
            must_keep.insert(is);
        }
        for ia in &cycle.anchors {
            for c in 0..(cycle.sequence.len()) {
                could_delete.insert(ia + c);
            }
        }
    }
    actions.retain(|a| !could_delete.contains(&a.id) || must_keep.contains(&a.id));
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test helper to build vector of Formatted Actions with IDs.
    fn build_actions(names: Vec<&str>) -> Vec<FormattedAction> {
        names
            .into_iter()
            .enumerate()
            .map(|(i, a)| FormattedAction {
                name: a.into(),
                id: i,
                ..FormattedAction::default()
            })
            .collect()
    }

    #[test]
    fn test_extract_cycles_one_dup() {
        let mut actions = build_actions(vec!["foo", "foo"]);
        assert_eq!(
            extract_cycles(&mut actions),
            vec![Cycle {
                id: 0,
                anchors: vec![0, 1],
                sequence: vec![1],
            }]
        );
    }

    // #[test]
    fn test_extract_cycles_longer() {
        let mut actions = build_actions(vec!["foo", "bar", "baz", "foo", "bar"]);
        assert_eq!(
            extract_cycles(&mut actions),
            vec![Cycle {
                id: 0,
                anchors: vec![0],
                sequence: vec![3,4],
            }]
        );
    }

    // #[test]
    fn test_decycle() {
        let mut actions = build_actions(vec!["foo", "bar", "baz", "foo", "bar"]);
        let cycles = extract_cycles(&actions);
        decycle(&mut actions, &cycles);
        let names: Vec<String> = actions.into_iter().map(|a| a.name).collect();
        assert_eq!(vec!["baz", "foo", "bar"], names);
    }

    #[test]
    fn test_decycle_one() {
        let mut actions = build_actions(vec!["foo", "foo", "foo"]);
        let cycles = extract_cycles(&actions);
        decycle(&mut actions, &cycles);
        let names: Vec<String> = actions.into_iter().map(|a| a.name).collect();
        assert_eq!(vec!["foo"], names);
    }
}
