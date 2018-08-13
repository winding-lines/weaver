use super::processor::{Column, Msg};
use api::Row;
use crossbeam_channel as channel;
use cursive::align::HAlign;
use cursive::theme::ColorStyle;
use cursive::Cursive;
use lib_goo::entities::RecommendReason;
use lib_tui::{ActionListView, ActionListViewItem};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum BasicColumn {
    Name,
    Detail,
}

/// The table view for the history.
impl ActionListViewItem<BasicColumn> for Row {
    fn to_column(&self, column: BasicColumn, is_focussed: bool) -> Option<String> {
        match *self {
            Row::Regular(ref r) => match column {
                BasicColumn::Name => Some(r.name.to_string()),
                BasicColumn::Detail => if is_focussed && r.location.is_some() {
                    Some(r.location.as_ref().unwrap().clone())
                } else {
                    None
                },
            },
            Row::Recommended(ref r) => match column {
                BasicColumn::Name => Some(r.name.to_string()),
                BasicColumn::Detail => match r.reason {
                    RecommendReason::CorrelatedMostRecent(_age) => Some("most recent".into()),
                    RecommendReason::CorrelatedMostFrequent(repeat) => {
                        Some(format!("most frequent {}", repeat))
                    }
                    _ => None,
                },
            },
        }
    }

    fn color_style(&self) -> Option<ColorStyle> {
        match self {
            Row::Recommended(_) => Some(ColorStyle::secondary()),
            Row::Regular(_) => Some(ColorStyle::primary()),
        }
    }
}

// An alias for the table view.
pub type TView = ActionListView<Row, BasicColumn>;

// Create the Cursive table for actions.
pub fn create_view(initial: Vec<Row>, processor_tx: &channel::Sender<Msg>) -> TView {
    let mut view = TView::new()
        .column(BasicColumn::Name, |c| {
            c.align(HAlign::Left).width_percent(70)
        })
        .column(BasicColumn::Detail, |c| c.align(HAlign::Right));

    // Select the current entry when 'enter' is pressed, then end the application.
    {
        let view_tx = processor_tx.clone();
        view.set_on_submit(
            move |siv: &mut Cursive, _row: usize, _column: usize, _index: usize| {
                view_tx.send(Msg::TableSubmit);

                siv.quit();
            },
        );
    }

    // Notify the UI that the selection is changed.
    {
        let view_tx = processor_tx.clone();
        view.set_on_select(
            move |siv: &mut Cursive, _row: usize, column: usize, index: usize| {
                if let Some(mut t) = siv.find_id::<TView>("actions") {
                    let value = t.borrow_item(index).cloned().map(|a| {
                        (
                            a,
                            if column == 0 {
                                Column::Left
                            } else {
                                Column::Right
                            },
                        )
                    });
                    view_tx.send(Msg::Selection(value));
                } else {
                    // Errors are harder to display in Cursive mode, also need to redirect stderr to file.
                    error!("cannot find table");
                }
            },
        );
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
