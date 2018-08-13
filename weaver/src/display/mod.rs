use lib_goo::config::OutputKind;
use lib_goo::entities::FormattedAction;

mod history_view;
pub mod main_screen;
mod output_selector;
mod processor;

pub struct UserSelection {
    pub action: Option<FormattedAction>,
    pub kind: Option<OutputKind>,
}
