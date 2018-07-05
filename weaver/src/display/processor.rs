use cursive::{CbFunc as CursiveCbFunc, Cursive};
use cursive::views::EditView;
use local_api;
use std::sync::{Arc, Mutex};
use chan;
use std::thread;
use super::{FormattedAction, table_view, UserSelection};
use super::output_selector;
use lib_goo::{config, FilteredVec};
use lib_goo::config::Destination;


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

    // Events from the annotation edit view
    AnnotationSubmit(Option<String>),

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
    pub env: Arc<config::Environment>,
    destination: Destination,
    table: FilteredVec,
    // current search/filter string
    search_string: Option<String>,
    cursive_sink: chan::Sender<Box<CursiveCbFunc>>,
    // A transmit channel to the Processors main loop
    self_tx: chan::Sender<Msg>,
}

impl Processor {
    fn _update_ui(&mut self) {
        // Build the content to display.
        let content = self.formatted_action.as_ref().map(|f| {
            let data = self.output_kind.lock().unwrap();
            f.clone().into_shell_command(&(*data).content, &self.env)
        }).unwrap_or_else(String::new);


        // Update the UI
        let update_command = move |siv: &mut Cursive| {
            if let Some(mut view) = siv.find_id::<EditView>("command") {
                view.set_content(content);
            };
        };
        self.cursive_sink.send(Box::new(update_command));
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

    // Handle a submit from the command edit view.
    fn submit_command(&mut self, f: Option<String>) {
        let name = f.unwrap_or_else(String::new);
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
    fn filter(&mut self, f: Option<&str>, selected_row: Option<usize>) {
        debug!("Received filter message {:?}", f);
        let content = self.table.filter(f);
        let tx = self.self_tx.clone();
        let update_table = move |siv: &mut Cursive| {
            if let Some(mut tview) = siv.find_id::<table_view::TView>("actions") {
                tview.clear();
                let select = content.len();
                tview.set_items(content);
                if select > 0 {
                    let index = selected_row.unwrap_or(select - 1);
                    tview.set_selected_row(index);
                    let selected = tview.borrow_item(index).cloned();

                    // Update the rest of the system with the selection.
                    // Since there are state changes need to defer to the processor.
                    tx.send(Msg::Selection(selected));
                }
            };
        };
        self.cursive_sink.send(Box::new(update_table));
    }

    fn set_selected_row(&mut self, row: usize) {
        let jump = move |siv: &mut Cursive| {
            if let Some(mut tview) = siv.find_id::<table_view::TView>("actions") {
                tview.set_selected_row(row);
            }
        };
        self.cursive_sink.send(Box::new(jump));
        let action = self.table.get(row);
        self.select_action(action);
    }

    fn jump_to_next(&mut self) {
        debug!("jumpToNext, search {:?} current {:?} ", self.search_string, self.formatted_action);
        let maybe_pos = match (self.search_string.as_ref(), self.formatted_action.as_ref()) {
            (Some(search), Some(action)) => {
                self.table.find_next(search, action.id - 1)
            }
            _ => None
        };
        if let Some(new_pos) = maybe_pos {
            self.set_selected_row(new_pos);
        }
    }

    fn jump_to_prev(&mut self) {
        debug!("jumpToPrev, search {:?} current {:?} ", self.search_string, self.formatted_action);
        let maybe_pos = match (self.search_string.as_ref(), self.formatted_action.as_ref()) {
            (Some(search), Some(action)) => {
                self.table.find_previous(search, action.id - 1)
            }
            _ => None
        };
        if let Some(new_pos) = maybe_pos {
            self.set_selected_row(new_pos);
        }
    }

    // Display the output selector UI with the current state.
    fn show_output_selector(&mut self) {
        let my_tx = self.self_tx.clone();
        let my_kind = Arc::clone(&self.output_kind);
        let show_kind = move |siv: &mut Cursive| {
            let k = my_kind.lock().unwrap();
            output_selector::show_output_selection(siv, &*k, &my_tx);
        };
        self.cursive_sink.send(Box::new(show_kind));
    }

    fn set_annotation(&self, annotation: &str) {
        if let Some(selection) = self.formatted_action.as_ref() {
            local_api::set_annotation(&self.destination, selection.id as u64, annotation)
                .expect("saving annotation");
        }
    }

    fn exit(&mut self) {
        self.cursive_sink.send(Box::new(|siv: &mut Cursive| {
            siv.quit();
        }));
    }
}

/// The selection_processor thread:
/// - owns the Table data, receives and processes filter events
/// - owns the current selections, receives and processes selection events
/// - refreshes the UI with the filtered data or selection
pub struct ProcessorThread {
    pub table: FilteredVec,
    pub kind: config::OutputKind,
    pub env: Arc<config::Environment>,
    pub destination: Destination,
    pub rx: chan::Receiver<Msg>,
    pub self_tx: chan::Sender<Msg>,
    pub tx: chan::Sender<UserSelection>,
    pub cursive_sink: chan::Sender<Box<CursiveCbFunc>>,
}


impl ProcessorThread {
    pub fn spawn(self)
              -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let mut processor = Processor {
                table: self.table,
                formatted_action: None,
                output_kind: Arc::new(Mutex::new(self.kind)),
                env: self.env,
                destination: self.destination.clone(),
                cursive_sink: self.cursive_sink,
                self_tx: self.self_tx,
                search_string: None,
            };

            // Process messages until done.
            loop {
                match self.rx.recv() {
                    Some(Msg::Selection(selection)) => {
                        debug!("Received selection message {:?}", selection);
                        processor.select_action(selection);
                    }

                    Some(Msg::TableSubmit(f)) => {
                        debug!("Exiting in TableSubmit, selection {:?}", f);
                        processor.select_action(f);
                    }

                    Some(Msg::FilterSubmit) => {
                        debug!("Exiting in FilterSubmit, selection {:?}", processor.formatted_action);
                        processor.exit();
                    }

                    Some(Msg::AnnotationSubmit(f)) => {
                        let input = f.as_ref().map(|s| s.as_str());
                        let content = input.unwrap_or("");
                        processor.set_annotation(content);
                    }

                    Some(Msg::CommandSubmit(f)) => {
                        // Handle a string submitted from the command box.
                        processor.submit_command(f);
                        debug!("Exiting in EditSubmit, selection {:?}", processor.formatted_action);
                        processor.exit();
                    }

                    Some(Msg::Filter(f)) => {
                        processor.filter(Some(f.as_str()), None);
                        processor.search_string = Some(f);
                    }
                    Some(Msg::JumpToSelection) => {
                        debug!("Received JumpToSelection");
                        let current_id = processor.formatted_action.as_ref().map(|a| a.id - 1);
                        processor.filter(None, current_id);
                    }
                    Some(Msg::JumpToNextMatch) => {
                        processor.jump_to_next();
                    }
                    Some(Msg::JumpToPrevMatch) => {
                        processor.jump_to_prev();
                    }
                    Some(Msg::ShowOutputSelector) => {
                        processor.show_output_selector();
                    }
                    Some(Msg::SelectKind(k)) => {
                        processor.select_kind(k);
                    }
                    Some(Msg::ExtractState) => {
                        debug!("Received end msg");
                        let mine = processor.output_kind.lock().unwrap();
                        self.tx.send(UserSelection {
                            action: processor.formatted_action,
                            kind: Some((*mine).clone()),
                        });
                        return;
                    }
                    None => debug!("received None message")
                }
            }
        })
    }
}

