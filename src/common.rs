pub const APP_NAME: &str = "CRZ Rough";

pub struct State {
    pub items_key: Vec<String>,
    pub item_values: Option<Vec<String>>,
    pub search_query: String,
    /// Only the indexes of items
    pub filtered_items: Vec<usize>,
    /// Position of the cursor in the filtered list (not the original list!)
    pub cursor_pos: usize,
    pub comment: Option<String>,
    /// if search fonctionnality is enabled or not
    pub search: bool,
    pub scroll_offset: usize,
    /// Cursor position within search_query (char index, not bytes)
    pub search_cursor: usize,
    /// Maximum number of list lines to display
    pub max_lines: usize,
    /// Actual visible lines reserved for the list
    pub max_visible: usize,
    /// Layout values computed once at init
    pub comment_lines: usize,
    pub search_lines: usize,
    pub total_lines: usize,
}

impl State {
    /// Get Item struct from index
    pub fn item(&self, index: usize) -> Item {
        Item { index, s: self }
    }

    /// Get an iterator over the filtered items as Item structs
    pub fn iter_filtered(&self) -> impl Iterator<Item = Item> {
        self.filtered_items.iter().map(|&idx| self.item(idx))
    }
}

pub struct Item<'a> {
    pub index: usize,
    pub s: &'a State,
}

impl<'a> Item<'a> {
    /// Get the label of the item (the key)
    pub fn label(&self) -> &str {
        &self.s.items_key[self.index]
    }

    /// Get the value of the item (the value if it exists, otherwise the key)
    pub fn value(&self) -> &str {
        match &self.s.item_values {
            Some(values) => &values[self.index],
            None => &self.s.items_key[self.index],
        }
    }
}

pub struct ListState {
    pub cursor_pos: usize,
    pub scroll_offset: usize,
    pub items_key: Vec<String>,
    pub item_values: Option<Vec<String>>,
    pub search_query: String,
    /// Only the indexes of items
    pub filtered_items: Vec<usize>,
    /// Actual visible lines reserved for the list
    pub max_visible: usize,
}

impl ListState {
    pub fn move_up(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos -= 1;
            // La ligne 0 du viewport est un "..." si scroll_offset > 0.
            if self.cursor_pos < self.scroll_offset {
                self.scroll_offset = self.cursor_pos;
            } else if self.scroll_offset > 0 && self.cursor_pos == self.scroll_offset {
                // Evite de poser le curseur sur la ligne "...".
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
        }
    }

    pub fn move_down(&mut self) {
        let max_idx = self.filtered_items.len().saturating_sub(1);
        if self.cursor_pos < max_idx {
            self.cursor_pos += 1;
            // La derniere ligne du viewport est un "..." si items depassent.
            let would_be_last = self.cursor_pos == self.scroll_offset + self.max_visible - 1;
            let more_below = self.scroll_offset + self.max_visible < self.filtered_items.len();
            if self.cursor_pos >= self.scroll_offset + self.max_visible {
                self.scroll_offset = self.cursor_pos + 1 - self.max_visible;
            } else if would_be_last && more_below {
                self.scroll_offset += 1;
            }
        }
    }
}
