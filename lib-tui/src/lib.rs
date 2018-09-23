//! An action list inspired from the cursive_table_view on the implementation side
//! and the file picker for VSCode on the visual side.

// Crate Dependencies ---------------------------------------------------------
extern crate cursive;
extern crate regex;
extern crate log;

mod action_list_view;

pub use self::action_list_view::{ActionListPos, ActionListView, ActionListViewItem};
