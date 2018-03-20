use ::errors::*;
use cursive::align::VAlign;
use cursive::Cursive;
use cursive::theme::{Color, PaletteColor, Theme};
use cursive::traits::*;
use cursive::views::{BoxView, Dialog, DummyView, EditView, LinearLayout, TextView};
pub use self::table::FormattedAction;
use std::sync::mpsc;
use std::time::Duration;
use termion::terminal_size;

mod table;
mod filter_processor;
mod selection_processor;

use self::filter_processor::FilterMsg;

// Create the Cursive environment.
fn create_cursive() -> Cursive {
    let mut siv = Cursive::new();

    // The global callback is triggered on the table
    siv.add_global_callback('q', |s| s.quit());

    // set our custom theme to match the terminal
    let theme = custom_theme_from_cursive(&siv);
    siv.set_theme(theme);

    // set the fps parameter to enable callbacks.
    siv.set_fps(60);

    siv
}

fn create_edit(tx: mpsc::Sender<FilterMsg>) -> EditView {
    EditView::new().on_edit(move |_: &mut Cursive, text: &str, _position: usize| {
        tx.send(FilterMsg::Filter(String::from(text)))
            .expect("send to filter");
    })
}

/// Display the content in a table and allows the user to exlore and select one of the options.
pub fn show(actions: Vec<FormattedAction>) -> Result<(Option<FormattedAction>)> {
    let mut siv = create_cursive();
    // Fill the screen
    let (width, height) = terminal_size().chain_err(|| "terminal size")?;
    let table_height = (height - 8) as usize;
    let table_width = (width - 6) as usize;

    // communication channels between views and data processor.
    let (submit_tx, submit_rx) = mpsc::channel();
    let (select_tx, select_rx) = mpsc::channel();
    let (edit_tx, edit_rx) = mpsc::channel();

    let mut table = table::Table::new(actions, table_height);
    let initial = table.filter(None);
    let filter_proc = filter_processor::create(table, edit_rx, siv.cb_sink().clone());
    let selection_proc = selection_processor::create(select_rx, siv.cb_sink().clone());

    // build the cursive scene
    let mut layout = LinearLayout::vertical();
    layout.add_child(table::create_view(initial,select_tx.clone(), submit_tx)
        .with_id("actions")
        .min_height(table_height)
        .min_width(table_width));
    layout.add_child(BoxView::with_fixed_size((0, 2), DummyView));
    let filter_pane = LinearLayout::horizontal()
        .child(TextView::new("Filter:    "))
        .child(create_edit(edit_tx.clone())
            .with_id("filter")
            .min_width(20))
        .child(TextView::new("     Ctrl-C to quit, <Enter> to run")
            .v_align(VAlign::Bottom));
    layout.add_child(filter_pane);
    let command_pane = LinearLayout::horizontal()
        .child(TextView::new("Selection: "))
        .child(EditView::new()
            .with_id("command")
            .min_width((width-15) as usize));
    layout.add_child(command_pane);
    siv.add_layer(
        Dialog::around(layout.min_size((70, height))).title("~ weaver ~")
    );

    siv.run();

    // Stop the filter processor
    edit_tx.send(FilterMsg::End)
        .expect("send end to filter");
    filter_proc.join()
        .expect("join filter thread");

    // Stop the selection processor
    select_tx.send(selection_processor::SelectionMsg::End)
        .expect("send end to select");
    selection_proc.join()
        .expect("join selection thread");


    if let Ok(selected) = submit_rx.recv_timeout(Duration::from_millis(100)) {
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