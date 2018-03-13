use ::errors::*;
use cursive::align::{HAlign, VAlign};
use cursive::Cursive;
use cursive::theme::{Color, PaletteColor, Theme};
use cursive::traits::*;
use cursive::views::{BoxView, Dialog, DummyView, EditView, LinearLayout, TextView};
use cursive_table_view::{TableView, TableViewItem};
use std::cmp::Ordering;
use std::sync::mpsc;
use std::time::Duration;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum BasicColumn {
    Index,
    Kind,
    Name,
}

#[derive(Clone, Debug)]
pub struct NameWithId {
    pub name: String,
    pub kind: String,
    pub id: usize,
}

impl TableViewItem<BasicColumn> for NameWithId {
    fn to_column(&self, column: BasicColumn) -> String {
        match column {
            BasicColumn::Name => self.name.to_string(),
            BasicColumn::Kind => self.kind.to_string(),
            BasicColumn::Index => format!("{}", self.id),
        }
    }

    fn cmp(&self, other: &Self, column: BasicColumn) -> Ordering where Self: Sized {
        match column {
            BasicColumn::Name => self.name.cmp(&other.name),
            BasicColumn::Kind => self.kind.cmp(&other.kind),
            BasicColumn::Index => self.id.cmp(&other.id),
        }
    }
}

// Create the Cursive environment.
fn create_cursive() -> Cursive {
    let mut siv = Cursive::new();
    siv.add_global_callback('q', |s| s.quit());
    let theme = custom_theme_from_cursive(&siv);
    siv.set_theme(theme);
    siv
}

// Create the table for actions.
fn create_table(labels: Vec<NameWithId>, tx: mpsc::Sender<Option<String>>) -> TableView<NameWithId, BasicColumn> {
    let mut table = TableView::<NameWithId, BasicColumn>::new()
        .column(BasicColumn::Index, "#", |c| c.width(6))
        .column(BasicColumn::Kind, "Kind", |c| c.align(HAlign::Left).width(6))
        .column(BasicColumn::Name, "Command", |c| c.align(HAlign::Left));

    let last = labels.len() - 1;
    table.set_items(labels);
    table.set_selected_row(last);

    // Select the current entry when 'enter' is pressed, then end the application.
    table.set_on_submit(move |siv: &mut Cursive, _row: usize, index: usize| {
        if let Some(mut t) = siv.find_id::<TableView<NameWithId, BasicColumn>>("actions") {
            let value = t.borrow_item(index).map(|s| s.name.clone());
            tx.send(value).unwrap();
        } else {
            // Errors are harder to display in Cursive mode, also need to redirect stderr to file.
            eprintln!("cannot find table");
        }

        /*
        siv.add_layer(
            Dialog::around(TextView::new(value))
                .title(format!("Removing row # {}", row))
                .button("Close", move |s| {
                    s.call_on_id("table", |table: &mut TableView<NameWithId, BasicColumn>| {
                        table.remove_item(index);
                    });
                    s.pop_layer()
                })
        );
        */
        siv.quit();
    });

    table
}

pub fn show(labels: Vec<NameWithId>) -> Result<(Option<String>)> {
    let mut siv = create_cursive();

    let (tx, rx) = mpsc::channel();

    let mut layout = LinearLayout::vertical();
    layout.add_child(create_table(labels, tx)
        .with_id("actions")
        .min_size((64, 35)));
    layout.add_child(BoxView::with_fixed_size((4, 0), DummyView));
    let vertical = LinearLayout::horizontal()
        .child(EditView::new().min_width(20))
        .child(TextView::new("Ctrl-C to quit\n<Enter> to run")
            .v_align(VAlign::Bottom));
    layout.add_child(vertical);
    siv.add_layer(
        Dialog::around(layout.min_size((70, 40))).title("~ weaver ~")
    );

    siv.run();
    if let Ok(selected) = rx.recv_timeout(Duration::from_millis(100)) {
        Ok(selected)
    } else {
        Ok(None)
    }
}

fn custom_theme_from_cursive(siv: &Cursive) -> Theme {
    // We'll return the current theme with a small modification.
    let mut theme = siv.current_theme().clone();

    theme.palette[PaletteColor::Background] = Color::TerminalDefault;

    theme
}