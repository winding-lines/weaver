use ::errors::*;
use cursive::align::VAlign;
use cursive::Cursive;
use cursive::event::Event;
use cursive::theme::{Color, PaletteColor, Theme};
use cursive::traits::*;
use cursive::views::{BoxView, Dialog, DummyView, EditView, LinearLayout, TextView};
use config::ActionKind;
use self::processor::Msg;
pub use self::table::FormattedAction;
use std::sync::mpsc;
use termion::terminal_size;

mod table;
mod processor;


pub struct UserSelection {
    pub action: Option<FormattedAction>,
    pub kind: Option<ActionKind>,
}

// Create the Cursive environment.
fn create_cursive() -> Cursive {
    let mut siv = Cursive::new();

    // set our custom theme to match the terminal
    let theme = custom_theme_from_cursive(&siv);
    siv.set_theme(theme);

    // set the fps parameter to enable callbacks.
    siv.set_fps(60);

    siv
}

fn send(tx: &mpsc::Sender<Msg>, msg: Msg) {
    match tx.send(msg) {
        Err(e) => error!("failed sending filter message {:?}", e),
        Ok(_) => ()
    }
}

fn create_filter_edit(tx: mpsc::Sender<Msg>) -> EditView {
    let tx2 = tx.clone();
    EditView::new()
        .on_edit(move |_: &mut Cursive, text: &str, _position: usize| {
            send(&tx, Msg::Filter(String::from(text)));
        })
        .on_submit(move |_: &mut Cursive, _content: &str| {
            send(&tx2, Msg::FilterSubmit);
        })
}

fn create_command_edit(tx: mpsc::Sender<Msg>) -> EditView {
    EditView::new()
        .on_submit(move |_: &mut Cursive, content: &str| {
            let message = Msg::CommandSubmit(Some(String::from(content)));
            send(&tx, message);
        })
}

/// Display the UI which allows the user to exlore and select one of the options.
pub fn show(actions: Vec<FormattedAction>, kind: ActionKind) -> Result<UserSelection> {
    // initialize cursive
    let mut siv = create_cursive();

    // Fill the screen
    let (width, height) = terminal_size().chain_err(|| "terminal size")?;
    let table_height = (height - 8) as usize;
    let table_width = (width - 7) as usize;

    // communication channels between views and data processor.
    let (process_tx, process_rx) = mpsc::channel::<processor::Msg>();

    // communication channel between processor and main function
    let (submit_tx, submit_rx) = mpsc::channel::<UserSelection>();


    let mut table = table::Table::new(actions, table_height);
    let initial = table.filter(None);
    let processor = processor::create(table,
                                      kind,
                                      process_rx,
                                      process_tx.clone(),
                                      submit_tx,
                                      siv.cb_sink().clone());

    // build the cursive scene
    let mut layout = LinearLayout::vertical();
    layout.add_child(table::create_view(initial, process_tx.clone())
        .with_id("actions")
        .min_height(table_height)
        .min_width(table_width));
    layout.add_child(BoxView::with_fixed_size((0, 2), DummyView));
    let filter_pane = LinearLayout::horizontal()
        .child(TextView::new("Filter:    "))
        .child(create_filter_edit(process_tx.clone())
            .with_id("filter")
            .min_width(20))
        .child(TextView::new("     Ctrl-C to quit, <Enter> to run")
            .v_align(VAlign::Bottom));
    layout.add_child(filter_pane);
    let command_pane = LinearLayout::horizontal()
        .child(TextView::new("Selection: "))
        .child(create_command_edit(process_tx.clone())
            .with_id("command")
            .min_width((width - 50) as usize));
    layout.add_child(command_pane);
    siv.add_layer(
        Dialog::around(layout.min_size((70, height)))
            .title("~ weaver ~")
        /*.button("Run", |_s| {})
        .button("Copy", |_s| {}) */);

    // WIP: getting Copy/Paste to work
    siv.add_global_callback(Event::AltChar('C'), |s| {
        error!("Entered Alt-C");
        s.quit()
    });

    // Do the initial display;
    process_tx.send(Msg::Filter(String::from(""))).expect("initial 'filter'");

    siv.focus_id("filter").expect("set focus on filter");

    siv.run();

    // Stop the filter processor
    send(&process_tx, processor::Msg::End);

    let user_selection = submit_rx.recv();
    let _ = processor.join();

    user_selection.chain_err(|| "could not receive final result")
}

fn custom_theme_from_cursive(siv: &Cursive) -> Theme {
    // We'll return the current theme with a small modification.
    let mut theme = siv.current_theme().clone();

    theme.palette[PaletteColor::Background] = Color::TerminalDefault;

    theme
}