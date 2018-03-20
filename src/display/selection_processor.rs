use cursive::{CbFunc as CursiveCbFunc, Cursive};
use cursive::views::EditView;
use std::sync::mpsc;
use std::thread;


/// Message types sent to the selection processor
pub enum SelectionMsg {
    End,
    Selection(Option<String>),
}


/// The selection_processor thread:
/// - receives and processes selection events.
/// - refreshes the UI with the selection.
pub fn create(rx: mpsc::Receiver<SelectionMsg>,
              cursive_sink: mpsc::Sender<Box<CursiveCbFunc>>)
              -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            match rx.recv() {
                Ok(SelectionMsg::Selection(f)) => {
                    let update_command = move |siv: &mut Cursive| {
                        if let Some(mut view) = siv.find_id::<EditView>("command") {
                            view.set_content(f.unwrap_or(String::from("")));
                        };
                    };
                    cursive_sink.send(Box::new(update_command))
                        .expect("send to command");
                }
                Ok(SelectionMsg::End) => {
                    return;
                }
                Err(e) => {
                    eprintln!("Error {:?} in selection thread", e);
                    return;
                }
            }
        }
    })
}


