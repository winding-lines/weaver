use lib_error::*;
use lib_goo::config::{net, Destination, Environment};
use lib_goo::entities::FormattedAction;
use lib_goo::entities::RecommendReason;
use lib_goo::filtered_vec::FilteredItem;
use lib_rpc::client as rpc_client;
use regex::Regex;
use std::path::Path;
use std::sync::Arc;

/// Enum used to represent historical and recommended actions for the UI.
#[derive(Clone, Debug)]
pub enum Row {
    Regular(FormattedAction),
    Recommended(FormattedAction),
}

impl Default for Row {
    fn default() -> Row {
        Row::Regular(FormattedAction::default())
    }
}

impl FilteredItem for Row {
    fn is_match(&self, regex: &Regex) -> bool {
        match *self {
            Row::Regular(ref fa) => regex.is_match(&fa.name),
            _ => true,
        }
    }
}

impl Row {
    /// Build the final list of actions by augmenting the historical ones with the recommended ones.
    fn build(initial: Vec<FormattedAction>) -> Vec<Row> {
        let mut rows = Vec::with_capacity(initial.len() + 1);
        for i in initial {
            let ui = match i.reason {
                RecommendReason::Historical => Row::Regular(i),
                _ => Row::Recommended(i),
            };
            rows.push(ui);
        }
        rows
    }
}

// Fetch recommendations for the given term.
pub fn fetch_recommendations(
    term: Option<String>,
    destination: &Destination,
    env: &Arc<Environment>,
) -> Result<Vec<Row>> {
    let net::PaginatedActions {
        entries: mut actions,
        total: _total,
        cycles: _cycles,
    } = rpc_client::recommendations(
        &destination,
        &net::RecommendationQuery {
            start: None,
            length: None,
            term,
        },
    )?;
    // rebase the command folders on the current work dir. This simplifies the UI interpretation.
    for mut a in actions.iter_mut() {
        if let Some(mut l) = a.location.as_mut() {
            let rebased = env.rebase(Path::new(&l).into())?;
            *l = Environment::encode_path(&rebased);
        }
    }
    // Put the most relevant and recent entries at the bottom.
    actions.reverse();
    Ok(Row::build(actions))
}
