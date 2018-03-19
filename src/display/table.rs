use cursive::align::HAlign;
use cursive::Cursive;
use cursive_table_view::{TableView, TableViewItem};
use std::cmp::Ordering;
use std::sync::mpsc;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum BasicColumn {
    Annotation,
    Index,
    Epic,
    Kind,
    Name,
}

#[derive(Clone, Debug)]
pub struct FormattedAction {
    pub annotation: Option<String>,
    pub id: usize,
    pub epic: Option<String>,
    pub kind: String,
    pub name: String,
}


impl TableViewItem<BasicColumn> for FormattedAction {
    fn to_column(&self, column: BasicColumn) -> String {
        match column {
            BasicColumn::Annotation => self.annotation.as_ref().map_or(String::from(""), |s| s.to_string()),
            BasicColumn::Index => format!("{}", self.id),
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
pub fn create_view(initial: Vec<FormattedAction>, tx: mpsc::Sender<Option<FormattedAction>>) -> TView {
    let mut view = TView::new()
        .column(BasicColumn::Index, "#", |c| c.width(6))
        .column(BasicColumn::Kind, " ", |c| c.align(HAlign::Left).width(1))
        .column(BasicColumn::Epic, "Epic", |c| c.align(HAlign::Left).width(6))
        .column(BasicColumn::Name, "Command", |c| c.align(HAlign::Left))
        .column(BasicColumn::Annotation, "Annotation", |c| c.align(HAlign::Left).width(10));

    // Select the current entry when 'enter' is pressed, then end the application.
    view.set_on_submit(move |siv: &mut Cursive, _row: usize, index: usize| {
        if let Some(mut t) = siv.find_id::<TView>("actions") {
            let value = t.borrow_item(index).map(|s| s.clone());
            tx.send(value).unwrap();
        } else {
            // Errors are harder to display in Cursive mode, also need to redirect stderr to file.
            eprintln!("cannot find table");
        }

        siv.quit();
    });

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
    filter: Option<String>,
    rows: usize,
}


impl Table {
    pub fn new(content: Vec<FormattedAction>, rows: usize) -> Table {
        Table {
            content,
            filter: None,
            rows,
        }
    }

    /// Build a vector with the subcomponents that match this filter.
    /// For None returns a new vector.
    pub fn filter(&mut self, filter: Option<String>) -> Vec<FormattedAction> {
        self.filter = filter;
        let mut content: Vec<FormattedAction> = Vec::new();


        if let Some(ref f) = self.filter {
            for entry in self.content.iter() {
                if entry.name.contains(f) || f.is_empty() {
                    content.push(entry.clone());
                }
            }
        } else {
            content.extend_from_slice(&self.content);
        }

        // Make sure content is at the bottom of the screen
        while content.len() < self.rows {
            content.insert(0, FormattedAction {
                annotation: None,
                id: 0,
                epic: None,
                kind: String::new(),
                name: String::new(),
            })
        };

        content
    }
}
