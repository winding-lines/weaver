use super::processor::{self, Msg};
use super::{history_view, Row, UserSelection};
use crossbeam_channel as channel;
use cursive::event::{Event, Key};
use cursive::theme::{Color, PaletteColor, Theme};
use cursive::traits::*;
use cursive::views::{BoxView, DummyView, EditView, LinearLayout, TextView};
use cursive::Cursive;
use lib_error::*;
use lib_goo::config::{Destination, Environment, OutputKind};
use lib_goo::entities::FormattedAction;
use lib_goo::filtered_vec::FilteredVec;
use std::sync::Arc;

const MARGIN_X: usize = 7;

// Create the Cursive environment.
fn create_cursive() -> Cursive {
    let mut siv = Cursive::default();

    // set our custom theme to match the terminal
    let theme = custom_theme_from_cursive(&siv);
    siv.set_theme(theme);

    // set the fps parameter to enable callbacks.
    siv.set_fps(10);

    siv
}

fn send(tx: &channel::Sender<Msg>, msg: Msg) {
    tx.send(msg);
}

fn create_filter_edit(tx: channel::Sender<Msg>) -> EditView {
    let tx2 = tx.clone();
    EditView::new()
        .on_edit(move |_: &mut Cursive, text: &str, _position: usize| {
            send(&tx, Msg::Filter(String::from(text)));
        })
        .on_submit(move |_: &mut Cursive, _content: &str| {
            send(&tx2, Msg::FilterSubmit);
        })
}

fn create_command_edit(tx: channel::Sender<Msg>) -> EditView {
    EditView::new().on_submit(move |_: &mut Cursive, content: &str| {
        let message = Msg::CommandSubmit(Some(String::from(content)));
        send(&tx, message);
    })
}

/*
fn create_annotation_edit(tx: channel::Sender<Msg>) -> EditView {
    EditView::new().on_submit(move |_: &mut Cursive, content: &str| {
        let message = Msg::AnnotationSubmit(Some(String::from(content)));
        send(&tx, message);
    })
}
*/

fn setup_global_keys(siv: &mut Cursive, ch: channel::Sender<Msg>) {
    let mapping = vec![
        (Event::CtrlChar('g'), Msg::JumpToSelection),
        (Event::CtrlChar('p'), Msg::JumpToPrevMatch),
        (Event::CtrlChar('n'), Msg::JumpToNextMatch),
    ];
    for (cursive_ev, processor_msg) in mapping {
        let my_ch = ch.clone();
        siv.add_global_callback(cursive_ev, move |_siv| {
            my_ch.send(processor_msg.clone());
        });
    }
    // Setup the ESC key to open up the Output Kind selector.
    siv.add_global_callback(Event::Key(Key::Esc), move |_s| {
        ch.send(Msg::ShowOutputSelector);
    });
}

/// Display the UI which allows the user to exlore and select one of the options.
pub fn display(
    actions: Vec<FormattedAction>,
    kind: &OutputKind,
    env: Arc<Environment>,
    destination: &Destination,
) -> Result<UserSelection> {
    debug!(
        "Entering main screen with {} actions, first one {:?}",
        actions.len(),
        actions.first()
    );
    // initialize cursive
    let mut siv = create_cursive();

    // Fill the screen
    let screen = siv.screen_size();
    let content_height = (screen.y - 9) as usize;

    // History table, the current margin constants found by experimentation.
    let history_height = content_height;
    let history_width = (screen.x - MARGIN_X) as usize;

    // communication channels between views and data processor.
    let (process_tx, process_rx) = channel::bounded(10);

    // communication channel between processor and main function
    let (submit_tx, submit_rx) = channel::bounded(10);

    // Build the main components: table and processor.
    let rows = Row::build(actions);
    let table = FilteredVec::new(rows, history_height);
    let initial = table.filter(None)?;

    let processor_thread = processor::ProcessorThread {
        table,
        kind: kind.clone(),
        env,
        destination: destination.clone(),
        rx: process_rx,
        self_tx: process_tx.clone(),
        tx: submit_tx,
        cursive_sink: siv.cb_sink().clone(),
    };

    let processor = processor_thread.spawn();

    // build the table pane
    let content = history_view::create_view(initial, &process_tx)
        .with_id("actions")
        .min_height(history_height)
        .fixed_width(history_width);

    // Assemble the table pane with the bottom fields
    let mut layout = LinearLayout::vertical();
    layout.add_child(content);
    layout.add_child(BoxView::with_fixed_size((0, 2), DummyView));

    // build the filter pane
    let filter_pane = LinearLayout::horizontal()
        .child(TextView::new("Filter:       "))
        .child(
            create_filter_edit(process_tx.clone())
                .with_id("filter")
                .min_width(20),
        );
    layout.add_child(filter_pane);

    // build the command pane
    let command_pane = LinearLayout::horizontal()
        .child(TextView::new("Selection:    "))
        .child(
            create_command_edit(process_tx.clone())
                .with_id("command")
                .min_width((screen.x - 50) as usize),
        );
    layout.add_child(command_pane);

    // build the annotation pane
    /*
    let annotation_pane = LinearLayout::horizontal()
        .child(TextView::new("Annotate:     "))
        .child(
            create_annotation_edit(process_tx.clone())
                .with_id("annotation")
                .min_width((screen.x - 50) as usize),
        );
    layout.add_child(annotation_pane);
    */

    // build the output kind UI
    let mut output_pane = LinearLayout::horizontal();
    output_pane.add_child(TextView::new(format!(
        "<Enter> will: {} {}| <Esc> to change | Ctrl-G to jump to selection",
        &kind.channel, &kind.content
    )));
    setup_global_keys(&mut siv, process_tx.clone());
    layout.add_child(output_pane);

    siv.add_layer(layout.min_size((screen.x, screen.y)));

    // Do the initial display;
    process_tx.send(Msg::Filter(String::from("")));

    siv.focus_id("filter").expect("set focus on filter");

    siv.run();

    // Stop the filter processor
    send(&process_tx, processor::Msg::ExtractState);

    let user_selection = submit_rx.recv();
    let _ = processor.join();

    // Extract the desired output kind, needs to be done in the same thread.
    user_selection.chain_err(|| "could not receive final result")
}

fn custom_theme_from_cursive(siv: &Cursive) -> Theme {
    // We'll return the current theme with a small modification.
    let mut theme = siv.current_theme().clone();

    theme.palette[PaletteColor::Background] = Color::TerminalDefault;

    theme
}
