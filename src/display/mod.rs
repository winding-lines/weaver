use cursive::Cursive;
use cursive::event::{Event, Key};
use cursive::theme::{Color, PaletteColor, Theme};
use cursive::traits::*;
use cursive::views::{BoxView, DummyView, EditView, LinearLayout, TextView};
use self::processor::Msg;
use std::sync::Arc;
use std::sync::mpsc;
use weaver_db::config::{Environment, OutputKind};
use weaver_db::entities::FormattedAction;
use weaver_db::RealStore;
use weaver_error::*;

mod table_view;
mod processor;
mod output_selector;


pub struct UserSelection {
    pub action: Option<FormattedAction>,
    pub kind: Option<OutputKind>,
}

// Create the Cursive environment.
fn create_cursive() -> Cursive {
    let mut siv = Cursive::default();

    // set our custom theme to match the terminal
    let theme = custom_theme_from_cursive(&siv);
    siv.set_theme(theme);

    // set the fps parameter to enable callbacks.
    siv.set_fps(60);

    siv
}

fn send(tx: &mpsc::Sender<Msg>, msg: Msg) {
    if let Err(e) = tx.send(msg) {
        error!("failed sending filter message {:?}", e)
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

fn create_annotation_edit(tx: mpsc::Sender<Msg>) -> EditView {
    EditView::new()
        .on_submit(move |_: &mut Cursive, content: &str| {
        let message = Msg::AnnotationSubmit(Some(String::from(content)));
        send(&tx, message);
    })
}

fn setup_global_keys(siv: &mut Cursive, ch: mpsc::Sender<Msg>) {
    let mapping = vec![
        (Event::CtrlChar('g'), Msg::JumpToSelection),
        (Event::CtrlChar('p'), Msg::JumpToPrevMatch),
        (Event::CtrlChar('n'), Msg::JumpToNextMatch),
    ];
    for (cursive_ev, processor_msg) in mapping {
        let my_ch = ch.clone();
        siv.add_global_callback(cursive_ev, move |_siv| {
            my_ch.send(processor_msg.clone()).expect("send JumpTo*");
        });
    }
    // Setup the ESC key to open up the Output Kind selector.
    siv.add_global_callback(Event::Key(Key::Esc), move |_s| {
        ch.send(Msg::ShowOutputSelector).expect("send ShowOutputSelector");
    });
}

/// Display the UI which allows the user to exlore and select one of the options.
pub fn main_screen(actions: Vec<FormattedAction>, kind: &OutputKind,
                   env: Arc<Environment>, store: Arc<RealStore>) -> Result<UserSelection> {
    // initialize cursive
    let mut siv = create_cursive();

    // Fill the screen
    let screen = siv.screen_size();
    // need to leave some space at the bottom, the current constant found by experimentation.
    let table_height = (screen.y - 9) as usize;
    let table_width = (screen.x - 7) as usize;

    // communication channels between views and data processor.
    let (process_tx, process_rx) = mpsc::channel::<processor::Msg>();

    // communication channel between processor and main function
    let (submit_tx, submit_rx) = mpsc::channel::<UserSelection>();


    // Build the main components: table and processor.
    let mut table = table_view::Table::new(actions, table_height);
    let initial = table.filter(None);

    let processor_thread = processor::ProcessorThread {
        table,
        kind: kind.clone(),
        env,
        real_store: store,
        rx: process_rx,
        self_tx: process_tx.clone(),
        tx: submit_tx,
        cursive_sink: siv.cb_sink().clone(),
    };

    let processor = processor_thread.spawn();


    // build the table pane
    let mut layout = LinearLayout::vertical();
    layout.add_child(table_view::create_view(initial, &process_tx)
        .with_id("actions")
        .min_height(table_height)
        .min_width(table_width));
    layout.add_child(BoxView::with_fixed_size((0, 2), DummyView));

    // build the filter pane
    let filter_pane = LinearLayout::horizontal()
        .child(TextView::new("Filter:       "))
        .child(create_filter_edit(process_tx.clone())
            .with_id("filter")
            .min_width(20));
    layout.add_child(filter_pane);

    // build the command pane
    let command_pane = LinearLayout::horizontal()
        .child(TextView::new("Selection:    "))
        .child(create_command_edit(process_tx.clone())
            .with_id("command")
            .min_width((screen.x - 50) as usize));
    layout.add_child(command_pane);


    // build the annotation pane
    let annotation_pane = LinearLayout::horizontal()
        .child(TextView::new("Annotate:     "))
        .child(create_annotation_edit(process_tx.clone())
            .with_id("annotation")
            .min_width((screen.x - 50 ) as usize));
    layout.add_child(annotation_pane);

    // build the output kind UI
    let mut output_pane = LinearLayout::horizontal();
    output_pane.add_child(TextView::new(format!("<Enter> will: {} {}| <Esc> to change | Ctrl-G to jump to selection", &kind.channel, &kind.content)));
    setup_global_keys(&mut siv, process_tx.clone());
    layout.add_child(output_pane);

    // build top level scene
    //siv.add_layer(
    //    Dialog::around(layout.min_size((width, height)))
    //       .padding((0, 0, 0, 0))
    //        .title("~ weaver ~"));
    siv.add_layer(
        layout.min_size((screen.x, screen.y)));

    // Do the initial display;
    process_tx.send(Msg::Filter(String::from(""))).expect("initial 'filter'");

    siv.focus_id("filter").expect("set focus on filter");

    siv.run();

    // Stop the filter processor
    send(&process_tx, processor::Msg::ExtractState);

    let user_selection = submit_rx.recv();
    let _ = processor.join();

    // Extract the desired output kind, needs to be done in the same thread.
    user_selection
        .chain_err(|| "could not receive final result")
}

fn custom_theme_from_cursive(siv: &Cursive) -> Theme {
    // We'll return the current theme with a small modification.
    let mut theme = siv.current_theme().clone();

    theme.palette[PaletteColor::Background] = Color::TerminalDefault;

    theme
}