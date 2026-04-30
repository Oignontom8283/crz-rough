use crate::common::State;

/// Insere `c` a la position `char_pos` dans `s`.
pub fn char_insert(s: &mut String, char_pos: usize, c: char) {
    let byte_pos = char_to_byte(s, char_pos);
    s.insert(byte_pos, c);
}

/// Supprime le char a gauche de `char_pos`, retourne le nouveau char_pos.
pub fn char_delete_left(s: &mut String, char_pos: usize) -> usize {
    if char_pos == 0 {
        return 0;
    }
    let byte_end = char_to_byte(s, char_pos);
    let byte_start = char_to_byte(s, char_pos - 1);
    s.drain(byte_start..byte_end);
    char_pos - 1
}

/// Supprime le mot entier a gauche du curseur (Ctrl+Backspace).
/// Retourne le nouveau char_pos.
pub fn delete_word_left(s: &mut String, char_pos: usize) -> usize {
    if char_pos == 0 {
        return 0;
    }
    let chars: Vec<char> = s.chars().collect();

    let mut pos = char_pos;
    while pos > 0 && chars[pos - 1] == ' ' {
        pos -= 1;
    }
    while pos > 0 && chars[pos - 1] != ' ' {
        pos -= 1;
    }

    let byte_start = char_to_byte(s, pos);
    let byte_end = char_to_byte(s, char_pos);
    s.drain(byte_start..byte_end);
    pos
}

pub fn update_filter(state: &mut State) {
    // Met a jour la liste filtree via le fuzzy match.
    let query = state.search_query.to_lowercase();
    state.filtered_items = (0..state.items_key.len())
        .filter(|&idx| fuzzy_match(&state.items_key[idx].to_lowercase(), &query))
        .collect();
}

fn fuzzy_match(text: &str, pattern: &str) -> bool {
    // Fuzzy simple: chaque char du pattern doit apparaitre dans l'ordre.
    if pattern.is_empty() {
        return true;
    }
    let mut chars = text.chars();
    for p in pattern.chars() {
        if !chars.any(|c| c == p) {
            return false;
        }
    }
    true
}

/// Convertit un indice de char en indice d'octet pour `s`.
fn char_to_byte(s: &str, char_pos: usize) -> usize {
    s.char_indices()
        .nth(char_pos)
        .map(|(b, _)| b)
        .unwrap_or(s.len())
}
