// STD Dependencies -----------------------------------------------------------
use std::cmp::{self, Ordering};
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

// External Dependencies ------------------------------------------------------
use cursive::align::HAlign;
use cursive::direction::Direction;
use cursive::event::{Callback, Event, EventResult, Key};
use cursive::theme::ColorStyle;
use cursive::vec::Vec2;
use cursive::view::{ScrollBase, View};
use cursive::With;
use cursive::{Cursive, Printer};

/// A trait for displaying and sorting items inside a
/// [`ActionListView`](struct.ActionListView.html).
pub trait ActionListViewItem<H>: Clone + Sized
where
    H: Eq + Hash + Copy + Clone + 'static,
{
    /// Method returning a string representation of the item for the
    /// specified column from type `H`.
    fn to_column(&self, column: H, is_focussed: bool) -> String;

    /// Method returning true if this item should be render as an important item.
    fn color_style(&self) -> Option<ColorStyle>
    where
        Self: Sized;
}

const HEIGHT_SUB: usize = 0;

pub struct ActionListView<T: ActionListViewItem<H>, H: Eq + Hash + Copy + Clone + 'static> {
    enabled: bool,
    scrollbase: ScrollBase,
    last_size: Vec2,

    columns: Vec<TableColumn<H>>,
    column_indicies: HashMap<H, usize>,

    focus: usize,
    items: Vec<T>,
    rows_to_items: Vec<usize>,

    // TODO Pass drawing offsets into the handlers so a popup menu
    // can be created easily?
    on_submit: Option<Rc<Fn(&mut Cursive, usize, usize)>>,
    on_select: Option<Rc<Fn(&mut Cursive, usize, usize)>>,
}

impl<T: ActionListViewItem<H>, H: Eq + Hash + Copy + Clone + 'static> ActionListView<T, H> {
    /// Creates a new empty `ActionListView` without any columns.
    ///
    /// A ActionListView should be accompanied by a enum of type `H` representing
    /// the table columns.
    pub fn new() -> Self {
        Self {
            enabled: true,
            scrollbase: ScrollBase::new(),
            last_size: Vec2::new(0, 0),

            columns: Vec::new(),
            column_indicies: HashMap::new(),

            focus: 0,
            items: Vec::new(),
            rows_to_items: Vec::new(),

            on_submit: None,
            on_select: None,
        }
    }

    /// Adds a column for the specified table colum from type `H` along with
    /// a title for its visual display.
    ///
    /// The provided callback can be used to further configure the
    /// created [`TableColumn`](struct.TableColumn.html).
    pub fn column<C: FnOnce(TableColumn<H>) -> TableColumn<H>>(
        mut self,
        column: H,
        callback: C,
    ) -> Self {
        self.column_indicies.insert(column, self.columns.len());
        self.columns.push(callback(TableColumn::new(column)));

        self
    }
    /// Disables this view.
    ///
    /// A disabled view cannot be selected.
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Re-enables this view.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Enable or disable this view.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Returns `true` if this view is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Sets a callback to be used when `<Enter>` is pressed while an item
    /// is selected.
    ///
    /// Both the currently selected row and the index of the corresponding item
    /// within the underlying storage vector will be given to the callback.
    ///
    /// # Example
    ///
    /// ```norun
    /// table.set_on_submit(|siv: &mut Cursive, row: usize, index: usize| {
    ///
    /// });
    /// ```
    pub fn set_on_submit<F>(&mut self, cb: F)
    where
        F: Fn(&mut Cursive, usize, usize) + 'static,
    {
        self.on_submit = Some(Rc::new(move |s, row, index| cb(s, row, index)));
    }

    /// Sets a callback to be used when `<Enter>` is pressed while an item
    /// is selected.
    ///
    /// Both the currently selected row and the index of the corresponding item
    /// within the underlying storage vector will be given to the callback.
    ///
    /// Chainable variant.
    ///
    /// # Example
    ///
    /// ```norun
    /// table.on_submit(|siv: &mut Cursive, row: usize, index: usize| {
    ///
    /// });
    /// ```
    pub fn on_submit<F>(self, cb: F) -> Self
    where
        F: Fn(&mut Cursive, usize, usize) + 'static,
    {
        self.with(|t| t.set_on_submit(cb))
    }

    /// Sets a callback to be used when an item is selected.
    ///
    /// Both the currently selected row and the index of the corresponding item
    /// within the underlying storage vector will be given to the callback.
    ///
    /// # Example
    ///
    /// ```norun
    /// table.set_on_select(|siv: &mut Cursive, row: usize, index: usize| {
    ///
    /// });
    /// ```
    pub fn set_on_select<F>(&mut self, cb: F)
    where
        F: Fn(&mut Cursive, usize, usize) + 'static,
    {
        self.on_select = Some(Rc::new(move |s, row, index| cb(s, row, index)));
    }

    /// Sets a callback to be used when an item is selected.
    ///
    /// Both the currently selected row and the index of the corresponding item
    /// within the underlying storage vector will be given to the callback.
    ///
    /// Chainable variant.
    ///
    /// # Example
    ///
    /// ```norun
    /// table.on_select(|siv: &mut Cursive, row: usize, index: usize| {
    ///
    /// });
    /// ```
    pub fn on_select<F>(self, cb: F) -> Self
    where
        F: Fn(&mut Cursive, usize, usize) + 'static,
    {
        self.with(|t| t.set_on_select(cb))
    }

    /// Removes all items from this view.
    pub fn clear(&mut self) {
        self.items.clear();
        self.rows_to_items.clear();
        self.focus = 0;
    }

    /// Returns the number of items in this table.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns `true` if this table has no items.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the index of the currently selected table row.
    pub fn row(&self) -> Option<usize> {
        if self.items.is_empty() {
            None
        } else {
            Some(self.focus)
        }
    }

    /// Selects the row at the specified index.
    pub fn set_selected_row(&mut self, row_index: usize) {
        self.focus = row_index;
        self.scrollbase.scroll_to(row_index);
    }

    /// Selects the row at the specified index.
    ///
    /// Chainable variant.
    pub fn selected_row(self, row_index: usize) -> Self {
        self.with(|t| t.set_selected_row(row_index))
    }

    /// Sets the contained items of the table.
    ///
    /// The currently active sort order is preserved and will be applied to all
    /// items.
    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.rows_to_items = Vec::with_capacity(self.items.len());

        for i in 0..self.items.len() {
            self.rows_to_items.push(i);
        }

        self.set_selected_row(0);
    }

    /// Sets the contained items of the table.
    ///
    /// The order of the items will be preserved even when the table is sorted.
    ///
    /// Chainable variant.
    pub fn items(self, items: Vec<T>) -> Self {
        self.with(|t| t.set_items(items))
    }

    /// Returns a immmutable reference to the item at the specified index
    /// within the underlying storage vector.
    pub fn borrow_item(&mut self, index: usize) -> Option<&T> {
        self.items.get(index)
    }

    /// Returns a mutable reference to the item at the specified index within
    /// the underlying storage vector.
    pub fn borrow_item_mut(&mut self, index: usize) -> Option<&mut T> {
        self.items.get_mut(index)
    }

    /// Returns a immmutable reference to the items contained within the table.
    pub fn borrow_items(&mut self) -> &Vec<T> {
        &self.items
    }

    /// Returns a mutable reference to the items contained within the table.
    ///
    /// Can be used to modify the items in place.
    pub fn borrow_items_mut(&mut self) -> &mut Vec<T> {
        &mut self.items
    }

    /// Returns the index of the currently selected item within the underlying
    /// storage vector.
    pub fn item(&self) -> Option<usize> {
        if self.items.is_empty() {
            None
        } else {
            Some(self.rows_to_items[self.focus])
        }
    }

    /// Selects the item at the specified index within the underlying storage
    /// vector.
    pub fn set_selected_item(&mut self, item_index: usize) {
        // TODO optimize the performance for very large item lists
        if item_index < self.items.len() {
            for (row, item) in self.rows_to_items.iter().enumerate() {
                if *item == item_index {
                    self.focus = row;
                    self.scrollbase.scroll_to(row);
                    break;
                }
            }
        }
    }

    /// Selects the item at the specified index within the underlying storage
    /// vector.
    ///
    /// Chainable variant.
    pub fn selected_item(self, item_index: usize) -> Self {
        self.with(|t| t.set_selected_item(item_index))
    }

    /// Inserts a new item into the table.
    ///
    /// The currently active sort order is preserved and will be applied to the
    /// newly inserted item.
    pub fn insert_item(&mut self, item: T) {
        self.items.push(item);
        self.rows_to_items.push(self.items.len());

        self.scrollbase.set_heights(
            self.last_size.y.saturating_sub(HEIGHT_SUB),
            self.rows_to_items.len(),
        );
    }

    /// Removes the item at the specified index within the underlying storage
    /// vector and returns it.
    pub fn remove_item(&mut self, item_index: usize) -> Option<T> {
        if item_index < self.items.len() {
            // Move the selection if the currently selected item gets removed
            if let Some(selected_index) = self.item() {
                if selected_index == item_index {
                    self.focus_up(1);
                }
            }

            // Remove the sorted reference to the item
            self.rows_to_items.retain(|i| *i != item_index);

            // Adjust remaining references
            for ref_index in &mut self.rows_to_items {
                if *ref_index > item_index {
                    *ref_index -= 1;
                }
            }

            // Update scroll height to prevent out of index drawing
            self.scrollbase.set_heights(
                self.last_size.y.saturating_sub(HEIGHT_SUB),
                self.rows_to_items.len(),
            );

            // Remove actual item from the underlying storage
            Some(self.items.remove(item_index))
        } else {
            None
        }
    }

    /// Removes all items from the underlying storage and returns them.
    pub fn take_items(&mut self) -> Vec<T> {
        self.scrollbase
            .set_heights(self.last_size.y.saturating_sub(HEIGHT_SUB), 0);
        self.set_selected_row(0);
        self.rows_to_items.clear();
        self.items.drain(0..).collect()
    }
}

impl<T: ActionListViewItem<H>, H: Eq + Hash + Copy + Clone + 'static> ActionListView<T, H> {
    fn draw_columns<C: Fn(&Printer, &TableColumn<H>)>(
        &self,
        printer: &Printer,
        sep: &str,
        callback: C,
    ) {
        let mut column_offset = 0;
        let column_count = self.columns.len();
        for (index, column) in self.columns.iter().enumerate() {
            let printer = &printer.offset((column_offset, 0)).cropped(printer.size);

            callback(printer, column);

            /* Do not draw columns for now.
            if index < column_count - 1 {
                printer.print((column.width + 1, 0), sep);
            }
            */

            column_offset += column.width + 3;
        }
    }

    fn draw_item(&self, printer: &Printer, i: usize, is_focussed: bool) {
        self.draw_columns(printer, "┆ ", |printer, column| {
            let value = self.items[self.rows_to_items[i]].to_column(column.column, is_focussed);
            column.draw_row(printer, value.as_str());
        });
    }

    fn focus_up(&mut self, n: usize) {
        self.focus -= cmp::min(self.focus, n);
    }

    fn focus_down(&mut self, n: usize) {
        self.focus = cmp::min(self.focus + n, self.items.len() - 1);
    }
}

impl<T: ActionListViewItem<H> + 'static, H: Eq + Hash + Copy + Clone + 'static> View
    for ActionListView<T, H>
{
    fn draw(&self, printer: &Printer) {
        let printer = &printer.cropped(printer.size);
        self.scrollbase.draw(printer, |printer, i| {
            let color = if i == self.focus {
                if !self.enabled && printer.focused {
                    ColorStyle::highlight()
                } else {
                    ColorStyle::highlight_inactive()
                }
            } else {
                self.items[self.rows_to_items[i]]
                    .color_style()
                    .unwrap_or_else(|| ColorStyle::primary())
            };

            printer.with_color(color, |printer| {
                self.draw_item(printer, i, i == self.focus);
            });
        });
    }

    fn layout(&mut self, size: Vec2) {
        if size == self.last_size {
            return;
        }

        let item_count = self.items.len();
        let column_count = self.columns.len();

        // Split up all columns into sized / unsized groups
        let (mut sized, mut usized): (Vec<&mut TableColumn<H>>, Vec<&mut TableColumn<H>>) =
            self.columns
                .iter_mut()
                .partition(|c| c.requested_width.is_some());

        // Subtract one for the seperators between our columns (that's column_count - 1)
        let mut available_width = size.x.saturating_sub(column_count.saturating_sub(1) * 3);

        // Reduce the with in case we are displaying a scrollbar
        if size.y.saturating_sub(1) < item_count {
            available_width = available_width.saturating_sub(2);
        }

        // Calculate widths for all requested columns
        let mut remaining_width = available_width;
        for column in &mut sized {
            column.width = match *column.requested_width.as_ref().unwrap() {
                TableColumnWidth::Percent(width) => cmp::min(
                    (size.x as f32 / 100.0 * width as f32).ceil() as usize,
                    remaining_width,
                ),
                TableColumnWidth::Absolute(width) => width,
            };
            remaining_width = remaining_width.saturating_sub(column.width);
        }

        // Spread the remaining with across the unsized columns
        let remaining_columns = usized.len();
        for column in &mut usized {
            column.width = (remaining_width as f32 / remaining_columns as f32).floor() as usize;
        }

        self.scrollbase
            .set_heights(size.y.saturating_sub(HEIGHT_SUB), item_count);
        self.last_size = size;
    }

    fn take_focus(&mut self, _: Direction) -> bool {
        self.enabled && !self.items.is_empty()
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        if !self.enabled {
            return EventResult::Ignored;
        }

        let last_focus = self.focus;
        match event {
            Event::Key(Key::Right) => {}
            Event::Key(Key::Left) => {}
            Event::Key(Key::Up) if self.focus > 0 => {
                self.focus_up(1);
            }
            Event::Key(Key::Down) if self.focus + 1 < self.items.len() => {
                self.focus_down(1);
            }
            Event::Key(Key::PageUp) => {
                self.focus_up(10);
            }
            Event::Key(Key::PageDown) => {
                self.focus_down(10);
            }
            Event::Key(Key::Home) => {
                self.focus = 0;
            }
            Event::Key(Key::End) => {
                self.focus = self.items.len() - 1;
            }
            Event::Key(Key::Enter) => {
                if !self.is_empty() && self.on_submit.is_some() {
                    let cb = self.on_submit.clone().unwrap();
                    let row = self.row().unwrap();
                    let index = self.item().unwrap();
                    return EventResult::Consumed(Some(Callback::from_fn(move |s| {
                        cb(s, row, index)
                    })));
                }
            }
            _ => return EventResult::Ignored,
        }

        let focus = self.focus;
        self.scrollbase.scroll_to(focus);

        if !self.is_empty() && last_focus != focus {
            let row = self.row().unwrap();
            let index = self.item().unwrap();
            EventResult::Consumed(
                self.on_select
                    .clone()
                    .map(|cb| Callback::from_fn(move |s| cb(s, row, index))),
            )
        } else {
            EventResult::Ignored
        }
    }
}

/// A type used for the construction of columns in a
/// [`ActionListView`](struct.ActionListView.html).
pub struct TableColumn<H: Copy + Clone + 'static> {
    column: H,
    alignment: HAlign,
    width: usize,
    default_order: Ordering,
    requested_width: Option<TableColumnWidth>,
}

enum TableColumnWidth {
    Percent(usize),
    Absolute(usize),
}

impl<H: Copy + Clone + 'static> TableColumn<H> {
    /// Sets the default ordering of the column.
    pub fn ordering(mut self, order: Ordering) -> Self {
        self.default_order = order;
        self
    }

    /// Sets the horizontal text alignment of the column.
    pub fn align(mut self, alignment: HAlign) -> Self {
        self.alignment = alignment;
        self
    }

    /// Sets how many characters of width this column will try to occupy.
    pub fn width(mut self, width: usize) -> Self {
        self.requested_width = Some(TableColumnWidth::Absolute(width));
        self
    }

    /// Sets what percentage of the width of the entire table this column will
    /// try to occupy.
    pub fn width_percent(mut self, width: usize) -> Self {
        self.requested_width = Some(TableColumnWidth::Percent(width));
        self
    }

    fn new(column: H) -> Self {
        Self {
            column,
            alignment: HAlign::Left,
            width: 0,
            default_order: Ordering::Less,
            requested_width: None,
        }
    }

    fn draw_row(&self, printer: &Printer, value: &str) {
        let value = match self.alignment {
            HAlign::Left => format!("{:<width$} ", value, width = self.width),
            HAlign::Right => format!("{:>width$} ", value, width = self.width),
            HAlign::Center => format!("{:^width$} ", value, width = self.width),
        };

        printer.print((0, 0), value.as_str());
    }
}
