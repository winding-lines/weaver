use ::errors::*;
use cursive::align::{HAlign, VAlign};
use cursive::Cursive;
use cursive::direction::Orientation;
use cursive::theme::{Color, PaletteColor, Theme};
use cursive::traits::*;
use cursive::views::{BoxView, Dialog, DummyView, LinearLayout, TextView};
use cursive_table_view::{TableView, TableViewItem};
use std::cmp::Ordering;
use std::sync::mpsc;
use std::time::Duration;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum BasicColumn {
    Index,
    Name,
}

#[derive(Clone, Debug)]
pub struct NameWithId {
    pub name: String,
    pub id: usize,
}

impl TableViewItem<BasicColumn> for NameWithId {
    fn to_column(&self, column: BasicColumn) -> String {
        match column {
            BasicColumn::Name => self.name.to_string(),
            BasicColumn::Index => format!("{}", self.id),
        }
    }

    fn cmp(&self, other: &Self, column: BasicColumn) -> Ordering where Self: Sized {
        match column {
            BasicColumn::Name => self.name.cmp(&other.name),
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
fn create_table(labels: Vec<NameWithId>, tx: mpsc::Sender<String>) -> TableView<NameWithId, BasicColumn> {
    let mut table = TableView::<NameWithId, BasicColumn>::new()
        .column(BasicColumn::Index, "#", |c| c.width(6))
        .column(BasicColumn::Name, "Command", |c| c.align(HAlign::Left));

    table.set_items(labels);

    table.set_on_submit(move |siv: &mut Cursive, row: usize, index: usize| {
        let value = siv.call_on_id("table", move |table: &mut TableView<NameWithId, BasicColumn>| {
            format!("{:?}", table.borrow_item(index).unwrap())
        }).unwrap();
        tx.send(value).unwrap();

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

    let mut layout = LinearLayout::new(Orientation::Horizontal);
    layout.add_child(create_table(labels, tx).min_size((32, 20)));
    layout.add_child(BoxView::with_fixed_size((4, 0), DummyView));
    layout.add_child(TextView::new("q to quit\nenter to select").v_align(VAlign::Center));
    siv.add_layer(
        Dialog::around(layout.min_size((50, 40))).title("Weaver")
    );

    siv.run();
    if let Ok(selected) = rx.recv_timeout(Duration::from_millis(100)) {
        Ok(Some(selected))
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