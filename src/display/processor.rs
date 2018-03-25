use cursive::{CbFunc as CursiveCbFunc, Cursive};
use cursive::views::{EditView};
use std::sync::mpsc;
use std::thread;
use super::{FormattedAction, table, UserSelection};


/// Message types sent to the selection processor
pub enum Msg {
    End,

    // Events from the table
    Selection(Option<FormattedAction>),
    TableSubmit(Option<FormattedAction>),

    // Events from the filter edit view
    Filter(String),
    FilterSubmit,

    // Events from the command edit view
    CommandSubmit(Option<String>),

}


/// The selection_processor thread:
/// - owns the Table data, receives and processes filter events
/// - owns the current selections, receives and processes selection events
/// - refreshes the UI with the filtered data or selection
pub fn create(mut table: table::Table,
              rx: mpsc::Receiver<Msg>,
              self_tx: mpsc::Sender<Msg>,
              tx: mpsc::Sender<UserSelection>,
              cursive_sink: mpsc::Sender<Box<CursiveCbFunc>>)
              -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut current_selection: Option<FormattedAction> = None;

        // Process messages until done.
        loop {
            match rx.recv() {
                Ok(Msg::Selection(f)) => {
                    debug!("Received selection message {:?}", f);

                    // Clone the name from reference
                    let name = f.as_ref().map(|s| s.name.clone()).unwrap_or(String::from(""));

                    // Consume now the parameter
                    current_selection = f;

                    // Update the UI
                    let update_command = move |siv: &mut Cursive| {
                        if let Some(mut view) = siv.find_id::<EditView>("command") {
                            view.set_content(name);
                        };
                    };
                    cursive_sink.send(Box::new(update_command))
                        .expect("send to command");
                }

                Ok(Msg::TableSubmit(f)) => {
                    current_selection = f;
                    debug!("Exiting in TableSubmit, selection {:?}", current_selection);
                    cursive_sink.send(Box::new(|siv: &mut Cursive| {
                        siv.quit();
                    })).expect("cursive send");
                }

                Ok(Msg::FilterSubmit) => {
                    debug!("Exiting in FilterSubmit, selection {:?}", current_selection);
                    cursive_sink.send(Box::new(|siv: &mut Cursive| {
                        siv.quit();
                    })).expect("cursive send");
                }

                Ok(Msg::CommandSubmit(f)) => {
                    let name = f.unwrap_or(String::from(""));

                    // Update current selection with the edited command
                    {
                        let mut sel = current_selection.get_or_insert(FormattedAction {
                            name: name.clone(),
                            kind: String::from("shell"),
                            id: 0,
                            annotation: None,
                            epic: None,
                        });
                        sel.name = name;
                    }


                    debug!("Exiting in EditSubmit, selection {:?}", current_selection);
                    cursive_sink.send(Box::new(|siv: &mut Cursive| {
                        siv.quit();
                    })).expect("cursive send");
                }

                Ok(Msg::Filter(f)) => {
                    debug!("Received filter message {:?}", f);
                    let content = table.filter(Some(f));
                    let tx = self_tx.clone();
                    let update_table = move |siv: &mut Cursive| {
                        if let Some(mut tview) = siv.find_id::<table::TView>("actions") {
                            tview.clear();
                            let select = content.len();
                            tview.set_items(content);
                            if select > 0 {
                                let index = select - 1;
                                tview.set_selected_row(index);
                                let selected = tview.borrow_item(index).map(|s| s.clone());

                                // Update the rest of the system with the selection.
                                // Since there are state changes need to defer to the processor.
                                tx.send(Msg::Selection(selected)).expect("send selection");
                            }
                        };
                    };
                    cursive_sink.send(Box::new(update_table))
                        .expect("send to update_table");
                }
                Ok(Msg::End) => {
                    debug!("Received end msg");
                    tx.send(UserSelection {
                        action: current_selection,
                        kind: None,
                    }).expect("send user selection");
                    return;
                }
                Err(e) => {
                    error!("Error {:?} in selection thread", e);
                    return;
                }
            }
        }
    })
}


