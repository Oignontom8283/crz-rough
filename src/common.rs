use std::borrow::Cow;


pub const APP_NAME: &str = "CRZ Rough";

pub struct State {
    pub items_key: Vec<String>,
    pub item_values: Option<Vec<String>>,
    pub search_query: String,
    /// Only the indexes of items
    pub filtered_items: Vec<usize>,
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


