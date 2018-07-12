//! An action list inspired from the cursive_table_view on the implementation side
//! and the file picker for VSCode on the visual side.

// Crate Dependencies ---------------------------------------------------------
extern crate cursive;

mod action_list_view;

pub use action_list_view::{ActionListView, ActionListViewItem};