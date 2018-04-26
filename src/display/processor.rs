use cursive::{CbFunc as CursiveCbFunc, Cursive};
use cursive::views::EditView;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;
use super::{FormattedAction, table, UserSelection};
use super::output_selector;
use weaver_db::config;


/// Message types sent to the selection processor
#[derive(Clone)]
pub enum Msg {
    ExtractState,

    // Events from the table
    Selection(Option<FormattedAction>),
    TableSubmit(Option<FormattedAction>),

    // Events from the filter edit view
    Filter(String),
    FilterSubmit,

    // Events from the Output Kind selection
    SelectKind(config::OutputKind),

    // Events from the command edit view
    CommandSubmit(Option<String>),

    // Global events
    ShowOutputSelector,
    JumpToSelection,
    JumpToPrevMatch,
    JumpToNextMatch,
}

/// State for the processor.
struct Processor {
    // current selected formatted action
    pub formatted_action: Option<FormattedAction>,
    // output_kind needs to be accessed from multiple threads.
    pub output_kind: Arc<Mutex<config::OutputKind>>,
    table: table::Table,
    cursive_sink: mpsc::Sender<Box<CursiveCbFunc>>,
    // A transmit channel to the Processors main loop
    self_tx: mpsc::Sender<Msg>,
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

    fn select_kind(&mut self, kind: config::OutputKind) {
        {
            let mut mine = self.output_kind.lock().unwrap();
            *mine = kind;
        }
        self._update_ui();
    }

    // Handle a submit from the coammand edit view.
    fn submit_command(&mut self, f: Option<String>) {
        let name = f.unwrap_or(String::from(""));
        let sel = self.formatted_action.get_or_insert(FormattedAction {
            name: name.clone(),
            kind: String::from("shell"),
            id: 0,
            annotation: None,
            epic: None,
            location: None,
        });
        sel.name = name;
    }

    // Filter the displayed commands to match the given string,
    // optionally selects the given row.
    fn filter(&mut self, f: Option<String>, selected_row: Option<usize>) {
        debug!("Received filter message {:?}", f);
        let content = self.table.filter(f);
        let tx = self.self_tx.clone();
        let update_table = move |siv: &mut Cursive| {
            if let Some(mut tview) = siv.find_id::<table::TView>("actions") {
                tview.clear();
                let select = content.len();
                tview.set_items(content);
                if select > 0 {
                    let index = selected_row.unwrap_or(select - 1);
                    tview.set_selected_row(index);
                    let selected = tview.borrow_item(index).map(|s| s.clone());

                    // Update the rest of the system with the selection.
                    // Since there are state changes need to defer to the processor.
                    tx.send(Msg::Selection(selected)).expect("send selection");
                }
            };
        };
        self.cursive_sink.send(Box::new(update_table))
            .expect("send to update_table");
    }

    fn jump_to_next(&mut self) {
        if let Some(ref _current) = self.formatted_action {
            unimplemented!();
        }
    }

    fn jump_to_prev(&mut self) {
        if let Some(ref _current) = self.formatted_action {
            unimplemented!();
        }
    }

    // Display the output selector UI with the current state.
    fn show_output_selector(&mut self) {
        let my_tx = self.self_tx.clone();
        let my_kind = Arc::clone(&self.output_kind);
        let show_kind = move |siv: &mut Cursive| {
            let k = my_kind.lock().unwrap();
            output_selector::show_output_selection(siv, (*k).clone(), my_tx);
        };
        self.cursive_sink.send(Box::new(show_kind))
            .expect("cursive send show kind");
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
pub fn create(table: table::Table,
              kind: config::OutputKind,
              rx: mpsc::Receiver<Msg>,
              self_tx: mpsc::Sender<Msg>,
              tx: mpsc::Sender<UserSelection>,
              cursive_sink: mpsc::Sender<Box<CursiveCbFunc>>)
              -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut processor = Processor {
            table,
            formatted_action: None,
            output_kind: Arc::new(Mutex::new(kind)),
            cursive_sink,
            self_tx,
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
                    processor.submit_command(f);
                    debug!("Exiting in EditSubmit, selection {:?}", processor.formatted_action);
                    processor.exit();
                }

                Ok(Msg::Filter(f)) => {
                    processor.filter(Some(f), None);
                }
                Ok(Msg::JumpToSelection) => {
                    debug!("Received JumpToSelection");
                    let current_id = processor.formatted_action.as_ref().map(|a| a.id - 1);
                    processor.filter(None, current_id);
                }
                Ok(Msg::JumpToNextMatch) => {
                    processor.jump_to_next();
                }
                Ok(Msg::JumpToPrevMatch) => {
                    processor.jump_to_prev();
                }
                Ok(Msg::ShowOutputSelector) => {
                    processor.show_output_selector();
                }
                Ok(Msg::SelectKind(k)) => {
                    processor.select_kind(k);
                }
                Ok(Msg::ExtractState) => {
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


