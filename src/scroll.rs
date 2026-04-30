use crate::common::State;

pub fn move_up(state: &mut State) {
    if state.cursor_pos > 0 {
        state.cursor_pos -= 1;
        // La ligne 0 du viewport est un "..." si scroll_offset > 0.
        if state.cursor_pos < state.scroll_offset {
            state.scroll_offset = state.cursor_pos;
        } else if state.scroll_offset > 0 && state.cursor_pos == state.scroll_offset {
            // Evite de poser le curseur sur la ligne "...".
            state.scroll_offset = state.scroll_offset.saturating_sub(1);
        }
    }
}

pub fn move_down(state: &mut State) {
    let max_idx = state.filtered_items.len().saturating_sub(1);
    if state.cursor_pos < max_idx {
        state.cursor_pos += 1;
        // La derniere ligne du viewport est un "..." si items depassent.
        let would_be_last = state.cursor_pos == state.scroll_offset + state.max_visible - 1;
        let more_below = state.scroll_offset + state.max_visible < state.filtered_items.len();
        if state.cursor_pos >= state.scroll_offset + state.max_visible {
            state.scroll_offset = state.cursor_pos + 1 - state.max_visible;
        } else if would_be_last && more_below {
            state.scroll_offset += 1;
        }
    }
}

pub fn clamp_list_cursor(state: &mut State) {
    // S'assure que le curseur reste dans les bornes de la liste filtree.
    let max_idx = state.filtered_items.len().saturating_sub(1);
    if state.cursor_pos > max_idx {
        state.cursor_pos = max_idx;
    }
    if state.scroll_offset > state.cursor_pos {
        state.scroll_offset = state.cursor_pos;
    }
}
