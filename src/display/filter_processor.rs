use cursive::{CbFunc as CursiveCbFunc, Cursive};
use std::sync::mpsc;
use std::thread;
use super::table;

/// Message types sent to the filter processor
pub enum FilterMsg {
    End,
    Filter(String),
}


/// The filter_processor thread:
/// - owns the Table data
/// - receives and processes filter requests.
/// - refreshes the table view with the new filtered content.
pub fn create(mut table: table::Table,
              rx: mpsc::Receiver<FilterMsg>,
              cursive_sink: mpsc::Sender<Box<CursiveCbFunc>>)
              -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            match rx.recv() {
                Ok(FilterMsg::Filter(f)) => {
                    let content = table.filter(Some(f));
                    let update_table = move |siv: &mut Cursive| {
                        if let Some(mut tview) = siv.find_id::<table::TView>("actions") {
                            table::redisplay(&mut tview, content);
                        };
                    };
                    cursive_sink.send(Box::new(update_table))
                        .expect("send to update_table");
                }
                Ok(FilterMsg::End) => {
                    return;
                }
                Err(e) => {
                    eprintln!("Error {:?} in filter thread", e);
                    return;
                }
            }
        }
    })
}


