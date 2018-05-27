use cursive::align::HAlign;
use cursive::Cursive;
use cursive_table_view::{TableView, TableViewItem};
use std::cmp::Ordering;
use std::sync::mpsc;
use super::processor::Msg;
use weaver_db::entities::FormattedAction;

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
pub fn create_view(initial: Vec<FormattedAction>, processor_tx: &mpsc::Sender<Msg>) -> TView {
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
                view_tx.send(Msg::TableSubmit(value)).expect("send submit");
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
                view_tx.send(Msg::Selection(value)).expect("send select");
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


/// Hold the data in the system, the view is managed by Cursive.
pub struct Table {
    content: Vec<FormattedAction>,
    rows: usize,
}


impl Table {
    pub fn new(content: Vec<FormattedAction>, rows: usize) -> Table {
        Table {
            content,
            rows,
        }
    }

    pub fn get(&self, i: usize) -> Option<FormattedAction> {
        self.content.get(i).cloned()
    }

    pub fn find_previous(&self, search: &str, current: usize)  -> Option<usize> {
        let size = self.content.len();

        for i in 1..size {
            // look through all the array, wrap around at the end
            let pos = if current >= i {
                current - i
            } else {
                current + size - i
            };

            if let Some(action) = self.content.get(pos) {
                if action.name.contains(search) {
                    return Some(pos);
                }
            }
        }
        None
    }

    pub fn find_next(&self, search: &str, current: usize)  -> Option<usize> {
        let size = self.content.len();

        for i in 1..size {
            // look through all the array, wrap around at the end
            let pos = if current + i < size  {
                current + i
            } else {
                current + i - size
            };

            if let Some(action) = self.content.get(pos) {
                if action.name.contains(search) {
                    return Some(pos);
                }
            }
        }
        None
    }

    /// Build a vector with the subcomponents that match this filter.
    /// For None returns a new vector.
    pub fn filter(&mut self, filter: Option<&str>) -> Vec<FormattedAction> {
        let mut content: Vec<FormattedAction> = Vec::new();


        if let Some(f) = filter {
            for entry in &self.content {
                if entry.name.contains(f)
                    || entry.epic.as_ref().map(|e| e.contains(f)).unwrap_or(false)
                    || f.is_empty() {
                    content.push(entry.clone());
                }
            }
        } else {
            content.extend_from_slice(&self.content);
        }

        // Make sure content is at the bottom of the screen
        while content.len() < self.rows {
            content.insert(0, FormattedAction::default() )
        };

        content
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prev() {
        let table = build_table();

        assert_eq!(Some(0), table.find_previous("abc",1));
        assert_eq!(None, table.find_previous("def", 1));
        assert_eq!(Some(1), table.find_previous("abc",0));
    }

    #[test]
    fn test_next() {
        let table = build_table();

        assert_eq!(Some(1), table.find_next("abc",0));
        assert_eq!(None, table.find_next("def", 1));
        // wrap around
        assert_eq!(Some(0), table.find_next("abc",1));
    }

    fn build_table() -> Table {
        Table::new(vec![
            FormattedAction {
                name: "abc".into(),
                id: 0,
                ..Default::default()
            },
            FormattedAction {
                name: "abcde".into(),
                id: 1,
                ..Default::default()
            },
        ], 2)
    }
}
