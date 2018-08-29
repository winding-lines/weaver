use lib_goo::entities::{ActionId, Cycle, FormattedAction};
use std::collections::{HashMap, HashSet};

// Estimate capacity for output containers where input duplicates are removed.
fn capacity(len: usize) -> usize {
    use std::cmp::max;
    max(
        len,
        if len < 20 {
            len
        } else {
            ((len as f32) * 0.8) as usize
        },
    )
}

// Assign an id to each action so that actions with the same name have the same
// IDs. The output is a vector where each position maps to the respective
// position of the passed in input.
pub(crate) fn assign_identity_ids<'a, I>(actions: I, len: usize) -> Vec<usize>
where
    I: Iterator<Item = &'a FormattedAction>,
{
    // Hashmap used to assign unique ids based on the identity of the object. In
    // the current implementation it's just the name.
    let mut identity_map = HashMap::with_capacity(capacity(len));
    // Monotonically increasing generator.
    let mut generator: usize = 0;
    // Output array.
    let mut out = Vec::with_capacity(capacity(len));
    for action in actions {
        // All the magic happens here in the entry() API.
        let id = identity_map.entry(&action.name).or_insert_with(|| {
            generator = generator + 1;
            generator
        });
        out.push(*id);
    }
    out
}

// Given an array of IDs find the cycles obtained by combining each id with the
// one just following it. If the input array contains cycles of length n the
// output array will contain IDs for cycles of length n + 1.
pub(crate) fn grow_cycle(current: &Vec<usize>) -> Vec<usize> {
    // Hashmap used to generate unique IDs
    let mut identity_map = HashMap::with_capacity(capacity(current.len()));
    // Monotonically increasing generator.
    let mut generator: usize = 0;
    // Output array
    let mut out = Vec::with_capacity(capacity(current.len()));
    // Setup an iterator with successive pairs.
    let first_it = current.iter();
    let second_it = current.iter().skip(1);
    for pair in first_it.zip(second_it) {
        let id = identity_map.entry(pair).or_insert_with(|| {
            generator = generator + 1;
            generator
        });
        out.push(*id);
    }
    out
}

// Represent positions of repeating anchors at the given level. All of these
// anchors are start of cycles with the length given by the level of this array.
#[derive(Default, Debug, PartialEq)]
struct Anchors(Vec<usize>);

impl Anchors {}

enum IdRepeats {
    // We have seen this ID once and this its first position
    Once(usize),
    // We have seen this ID multiple times and it is part of an Anchors object,
    // this is the cycle index.
    Repeat(usize),
}

// Extract the position of anchors (repeating ids) in this array.
fn extract_anchors(ids: &[usize]) -> Vec<Anchors> {
    use std::collections::hash_map::Entry;

    let mut counts: HashMap<usize, IdRepeats> = HashMap::new();
    let mut anchors: Vec<Anchors> = Vec::new();

    for (pos, id) in ids.iter().enumerate() {
        match counts.entry(*id) {
            Entry::Vacant(ve) => {
                ve.insert(IdRepeats::Once(pos));
            }
            Entry::Occupied(mut oe) => match *oe.get() {
                IdRepeats::Once(first) => {
                    let id = anchors.len();
                    anchors.push(Anchors(vec![first, pos]));
                    *oe.get_mut() = IdRepeats::Repeat(id);
                }
                IdRepeats::Repeat(id) => {
                    anchors[id].0.push(pos);
                }
            },
        }
    }

    return anchors;
}

// Take a Cycle containing indexes in the level index and translate it to
// positions in the lower level.
fn anchor_to_cycle(anchor: &Anchors, actions: &[FormattedAction], cycle_len: usize) -> Cycle {
    let anchors: Vec<ActionId> = anchor.0.iter().map(|a| actions[*a].id.clone()).collect();
    let last = *anchor.0.last().unwrap();
    let sequence: Vec<ActionId> = actions[last..last + cycle_len]
        .iter()
        .map(|a| a.id.clone())
        .collect();
    Cycle {
        id: 0,
        anchors,
        sequence,
    }
}

fn ids_to_cycles(ids: &[usize], cycle_len: usize, actions: &[FormattedAction], start: usize) -> Vec<Cycle> {
    let mut out = Vec::new();
    for seq in extract_anchors(ids).iter() {
        let mut cycle = anchor_to_cycle(seq, actions, cycle_len);
        cycle.id = start + out.len();
        out.push(cycle)
    }
    Cycle::remove_overlapping(&mut out);
    out
}

// Public function extracting cycles. It assigns IDs based on the desired
// equivalence (name). then builds an array for each cycle len desired based on
// the layer below.
pub fn extract_cycles(actions: &Vec<FormattedAction>, limit: usize) -> Vec<Cycle> {
    let mut cycles: Vec<Cycle> = Vec::new();
    let mut previous = assign_identity_ids(actions.iter(), actions.len());
    let start = cycles.len();
    cycles.append(&mut ids_to_cycles(&previous, 1, actions, start));

    let mut ids: Vec<Vec<usize>> = Vec::new();
    for cycle_len in 2..(limit + 1) {
        let cycle_ids = grow_cycle(&previous);
        ids.push(previous);
        previous = cycle_ids;
        let start = cycles.len();
        cycles.append(&mut ids_to_cycles(&previous, cycle_len, actions, start));
    }

    cycles
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
            let mut repeating = ia.clone();
            for _c in 0..(cycle.sequence.len()) {
                let next = repeating.next();
                could_delete.insert(repeating);
                repeating = ActionId::new(next);
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
                id: ActionId(i),
                ..FormattedAction::default()
            })
            .collect()
    }

    #[test]
    fn test_extract_cycles_of_length_1() {
        let mut actions = build_actions(vec!["foo", "foo"]);
        assert_eq!(
            extract_cycles(&mut actions, 2),
            vec![Cycle {
                id: 0,
                anchors: vec![0, 1],
                sequence: vec![1],
            }]
        );
        let mut actions = build_actions(vec!["foo", "foo", "foo"]);
        assert_eq!(
            extract_cycles(&mut actions, 2),
            vec![Cycle {
                id: 0,
                anchors: vec![0, 1, 2],
                sequence: vec![1],
            }]
        );
    }

    #[test]
    fn test_extract_cycles() {
        let mut actions = build_actions(vec!["foo", "bar", "baz", "foo", "bar"]);
        assert_eq!(
            extract_cycles(&mut actions, 2),
            vec![
                Cycle {
                    id: 0,
                    sequence: vec![3],
                    anchors: vec![0, 3],
                },
                Cycle {
                    id: 1,
                    sequence: vec![4],
                    anchors: vec![1, 4],
                },
                Cycle {
                    id: 2,
                    anchors: vec![0, 3],
                    sequence: vec![3, 4],
                },
            ]
        );
    }

    #[test]
    fn test_assign_ids_one() {
        let actions = build_actions(vec!["foo", "foo"]);
        let ids = assign_identity_ids(actions.iter(), actions.len());
        assert_eq!(vec![1, 1], ids);
        let actions = build_actions(vec!["foo", "foo", "foo"]);
        let ids = assign_identity_ids(actions.iter(), actions.len());
        assert_eq!(vec![1, 1, 1], ids);
    }

    #[test]
    fn test_assign_ids() {
        let actions = build_actions(vec!["foo", "bar", "baz", "foo", "bar"]);
        let ids = assign_identity_ids(actions.iter(), actions.len());
        assert_eq!(vec![1, 2, 3, 1, 2], ids);
    }

    #[test]
    fn test_grow() {
        let current = vec![1, 2, 3, 1, 2];
        let next = grow_cycle(&current);
        assert_eq!(vec![1, 2, 3, 1], next);
        let next2 = grow_cycle(&next);
        assert_eq!(vec![1, 2, 3], next2);
    }

    #[test]
    fn test_extract_anchors() {
        let current = vec![1, 2, 3, 1, 2, 1];
        let ac = extract_anchors(&current);
        assert_eq!(ac, vec![Anchors(vec![0, 3, 5]), Anchors(vec![1, 4])])
    }

    #[test]
    fn test_decycle_one() {
        let mut actions = build_actions(vec!["foo", "foo", "foo"]);
        let cycles = extract_cycles(&actions, 3);
        decycle(&mut actions, &cycles);
        let names: Vec<String> = actions.into_iter().map(|a| a.name).collect();
        assert_eq!(vec!["foo"], names);
    }

    #[test]
    fn test_decycle() {
        let mut actions = build_actions(vec!["foo", "bar", "baz", "foo", "bar"]);
        let cycles = extract_cycles(&actions, 2);
        decycle(&mut actions, &cycles);
        let names: Vec<String> = actions.into_iter().map(|a| a.name).collect();
        assert_eq!(vec!["baz", "foo", "bar"], names);
    }

}
