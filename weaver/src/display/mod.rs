use lib_ai::recommender;
use lib_goo::config::OutputKind;
use lib_goo::entities::FormattedAction;
use lib_goo::filtered_vec::FilteredItem;
use regex::Regex;

mod history_view;
pub mod main_screen;
mod output_selector;
mod processor;

pub struct UserSelection {
    pub action: Option<FormattedAction>,
    pub kind: Option<OutputKind>,
}

/// Enum used to represent historical and recommended actions for the UI.
#[derive(Clone, Debug)]
pub enum Row {
    Regular(FormattedAction),
    Recommended(FormattedAction),
    Separator,
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
        let recommended = recommender::recommend(&initial);
        debug!(
            "Got {} recommended entries, first {:?}",
            recommended.len(),
            recommended.first()
        );
        let mut rows = Vec::with_capacity(initial.len() + recommended.len() + 1);
        for i in initial {
            rows.push(Row::Regular(i));
        }
        rows.push(Row::Separator);
        for i in recommended {
            rows.push(Row::Recommended(i));
        }
        rows
    }
}
