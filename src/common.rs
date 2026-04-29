
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