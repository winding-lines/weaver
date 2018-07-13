use super::processor::Msg;
use super::Row;
/// The table view for the history.
use chan;
use cursive::align::HAlign;
use cursive::theme::ColorStyle;
use cursive::Cursive;
use lib_tui::{ActionListView, ActionListViewItem};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum BasicColumn {
    Index,
    Name,
}

impl ActionListViewItem<BasicColumn> for Row {
    fn to_column(&self, column: BasicColumn) -> String {
        match *self {
            Row::Regular(ref r) => match column {
                BasicColumn::Index => if r.id != 0 {
                    format!("{}", r.id)
                } else {
                    String::new()
                },
                BasicColumn::Name => r.name.to_string(),
            },
            Row::Recommended(ref r) => match column {
                BasicColumn::Index => if r.id != 0 {
                    format!("{}", r.id)
                } else {
                    String::new()
                },
                BasicColumn::Name => r.name.to_string()
            },
            Row::Separator => match column {
                BasicColumn::Index => String::new(),
                BasicColumn::Name => "----\\ Recommended /-----".to_string(),
            }
        }
    }

    fn color_style(&self) -> Option<ColorStyle> {
        match self {
            Row::Recommended(_) => Some(ColorStyle::secondary()),
            Row::Regular(_) => Some(ColorStyle::primary()),
            Row::Separator => Some(ColorStyle::secondary()),
        }
    }
}

// An alias for the table view.
pub type TView = ActionListView<Row, BasicColumn>;

// Create the Cursive table for actions.
pub fn create_view(initial: Vec<Row>, processor_tx: &chan::Sender<Msg>) -> TView {
    let mut view = TView::new()
        //.column(BasicColumn::Index, |c| c.width(6))
        .column(BasicColumn::Name, |c| c.align(HAlign::Left))
        ;

    debug!("Entering create_view with {} entries", initial.len());
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
pub fn redisplay(view: &mut TView, content: Vec<Row>) {
    view.clear();
    let select = content.len();
    view.set_items(content);
    if select > 0 {
        view.set_selected_row(select - 1);
    }
}
