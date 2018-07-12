/// Display the list of recommended actions based on the current status.
use chan;
use cursive::align::HAlign;
use lib_tui::{ActionListView, ActionListViewItem};
use std::cmp::Ordering;
use super::processor::Msg;
use lib_ai::recommender::RecommendedAction;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Column {
    Name,
}

impl ActionListViewItem<Column> for RecommendedAction {
    fn to_column(&self, column: Column) -> String {
        match column {
            Column::Name => self.name.to_string(),
        }
    }

    fn cmp(&self, other: &Self, column: Column) -> Ordering where Self: Sized {
        match column {
            Column::Name => self.name.cmp(&other.name),
        }
    }
}

// An alias for the table view.
pub type TView = ActionListView<RecommendedAction, Column>;


// Create the Cursive table for actions.
pub fn create_view(initial: Vec<RecommendedAction>, _processor_tx: &chan::Sender<Msg>) -> TView {
    let mut view = TView::new()
        .column(Column::Name, "Recommended", |c| c.align(HAlign::Left));
    redisplay(&mut view, initial);
    view
}

// Display and redisplay the content, for example when the filter changes.
pub fn redisplay(view: &mut TView, content: Vec<RecommendedAction>) {
    view.clear();
    let select = content.len();
    view.set_items(content);
    if select > 0 {
        view.set_selected_row(select - 1);
    }
}
