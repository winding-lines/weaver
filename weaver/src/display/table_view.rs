use chan;
use cursive::align::HAlign;
use cursive::Cursive;
use cursive_table_view::{TableView, TableViewItem};
use std::cmp::Ordering;
use super::processor::Msg;
use lib_goo::entities::FormattedAction;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum BasicColumn {
    Annotation,
    Index,
    Epic,
    Kind,
    Name,
}

impl TableViewItem<BasicColumn> for FormattedAction {
    fn to_column(&self, column: BasicColumn) -> String {
        match column {
            BasicColumn::Annotation => self.annotation.as_ref().map_or(String::from(""), |s| s.to_string()),
            BasicColumn::Index => if self.id != 0 { format!("{}", self.id) } else { String::from("") },
            BasicColumn::Epic => self.epic.as_ref().map_or(String::from(""), |s| s.to_string()),
            BasicColumn::Kind => self.kind.to_string(),
            BasicColumn::Name => self.name.to_string(),
        }
    }

    fn cmp(&self, other: &Self, column: BasicColumn) -> Ordering where Self: Sized {
        match column {
            BasicColumn::Annotation => self.annotation.cmp(&other.annotation),
            BasicColumn::Index => self.id.cmp(&other.id),
            BasicColumn::Epic => self.epic.cmp(&other.epic),
            BasicColumn::Kind => self.kind.cmp(&other.kind),
            BasicColumn::Name => self.name.cmp(&other.name),
        }
    }
}

// An alias for the table view.
pub type TView = TableView<FormattedAction, BasicColumn>;

// Create the Cursive table for actions.
pub fn create_view(initial: Vec<FormattedAction>, processor_tx: &chan::Sender<Msg>) -> TView {
    let mut view = TView::new()
        .column(BasicColumn::Index, "#", |c| c.width(6))
        .column(BasicColumn::Kind, " ", |c| c.align(HAlign::Left).width(1))
        .column(BasicColumn::Epic, "Epic", |c| c.align(HAlign::Left).width(6))
        .column(BasicColumn::Name, "Command", |c| c.align(HAlign::Left))
        .column(BasicColumn::Annotation, "Annotation", |c| c.align(HAlign::Left).width(10));

    // Select the current entry when 'enter' is pressed, then end the application.
    {
        let view_tx = processor_tx.clone();
        view.set_on_submit(move |siv: &mut Cursive, _row: usize, index: usize| {
            if let Some(mut t) = siv.find_id::<TView>("actions") {
                let value = t.borrow_item(index).cloned();
                view_tx.send(Msg::TableSubmit(value));
            } else {
                error!("cannot find table");
            }

            siv.quit();
        });
    }

    // Notify the UI that the selection is changed.
    {
        let view_tx = processor_tx.clone();
        view.set_on_select(move |siv: &mut Cursive, _row: usize, index: usize| {
            if let Some(mut t) = siv.find_id::<TView>("actions") {
                let value = t.borrow_item(index).cloned();
                view_tx.send(Msg::Selection(value));
            } else {
                // Errors are harder to display in Cursive mode, also need to redirect stderr to file.
                error!("cannot find table");
            }
        });
    }
    redisplay(&mut view, initial);

    view
}

// Display and redisplay the content, for example when the filter changes.
pub fn redisplay(view: &mut TView, content: Vec<FormattedAction>) {
    view.clear();
    let select = content.len();
    view.set_items(content);
    if select > 0 {
        view.set_selected_row(select - 1);
    }
}


