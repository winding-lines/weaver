use config;
use cursive::{CbFunc as CursiveCbFunc, Cursive};
use cursive::views::EditView;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;
use super::{FormattedAction, table, UserSelection};
use super::output_kind;


/// Message types sent to the selection processor
pub enum Msg {
    End,

    // Events from the table
    Selection(Option<FormattedAction>),
    TableSubmit(Option<FormattedAction>),

    // Events from the filter edit view
    Filter(String),
    FilterSubmit,

    // Events from the Output Kind selection
    ShowKind,
    SelectKind(config::OutputKind),

    // Events from the command edit view
    CommandSubmit(Option<String>),

}

/// State for the processor.
struct Processor {
    pub formatted_action: Option<FormattedAction>,
    // output_kind needs to be accessed from multiple threads.
    pub output_kind: Arc<Mutex<config::OutputKind>>,
    pub cursive_sink: mpsc::Sender<Box<CursiveCbFunc>>,
}

impl Processor {

    fn _update_ui(&mut self) {
        // Build the content to display.
        let content = self.formatted_action.as_ref().map(|f| {
            let data = self.output_kind.lock().unwrap();
            f.clone().as_shell_command(&(*data).content)
        }).unwrap_or(String::from(""));


        // Update the UI
        let update_command = move |siv: &mut Cursive| {
            if let Some(mut view) = siv.find_id::<EditView>("command") {
                view.set_content(content);
            };
        };
        self.cursive_sink.send(Box::new(update_command))
            .expect("send to command");

    }

    fn select_action(&mut self, selection: Option<FormattedAction>) {
        self.formatted_action = selection;
        self._update_ui();

    }

    fn select_kind(&mut self, kind: config::OutputKind ) {
        {
            let mut mine = self.output_kind.lock().unwrap();
            *mine = kind;
        }
        self._update_ui();
    }

    fn exit(&mut self) {
        self.cursive_sink.send(Box::new(|siv: &mut Cursive| {
            siv.quit();
        })).expect("cursive send");
    }
}


/// The selection_processor thread:
/// - owns the Table data, receives and processes filter events
/// - owns the current selections, receives and processes selection events
/// - refreshes the UI with the filtered data or selection
pub fn create(mut table: table::Table,
              kind: config::OutputKind,
              rx: mpsc::Receiver<Msg>,
              self_tx: mpsc::Sender<Msg>,
              tx: mpsc::Sender<UserSelection>,
              cursive_sink: mpsc::Sender<Box<CursiveCbFunc>>)
              -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut processor = Processor {
            formatted_action: None,
            output_kind: Arc::new(Mutex::new(kind)),
            cursive_sink,
        };

        // Process messages until done.
        loop {
            match rx.recv() {
                Ok(Msg::Selection(selection)) => {
                    debug!("Received selection message {:?}", selection);
                    processor.select_action(selection);
                }

                Ok(Msg::TableSubmit(f)) => {
                    debug!("Exiting in TableSubmit, selection {:?}", f);
                    processor.select_action(f);
                }

                Ok(Msg::FilterSubmit) => {
                    debug!("Exiting in FilterSubmit, selection {:?}", processor.formatted_action);
                    processor.exit();
                }

                Ok(Msg::CommandSubmit(f)) => {
                    // Handle a string submitted from the command box.
                    let name = f.unwrap_or(String::from(""));

                    {
                        let mut sel = processor.formatted_action.get_or_insert(FormattedAction {
                            name: name.clone(),
                            kind: String::from("shell"),
                            id: 0,
                            annotation: None,
                            epic: None,
                            location: None,
                        });
                        sel.name = name;
                    }


                    debug!("Exiting in EditSubmit, selection {:?}", processor.formatted_action);
                    processor.exit();
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
                    processor.cursive_sink.send(Box::new(update_table))
                        .expect("send to update_table");
                }
                Ok(Msg::ShowKind) => {
                    let my_tx = self_tx.clone();
                    let my_kind = Arc::clone(&processor.output_kind);
                    let show_kind = move |siv: &mut Cursive| {
                        let k = my_kind.lock().unwrap();
                        output_kind::show_output_selection(siv, (*k).clone(), my_tx);
                    };
                    processor.cursive_sink.send(Box::new(show_kind))
                        .expect("cursive send show kind");
                }
                Ok(Msg::SelectKind(k)) => {
                    processor.select_kind(k);
                }
                Ok(Msg::End) => {
                    debug!("Received end msg");
                    let mine = processor.output_kind.lock().unwrap();
                    tx.send(UserSelection {
                        action: processor.formatted_action,
                        kind: Some((*mine).clone()),
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


