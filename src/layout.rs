use crate::common::State;

pub fn init_layout(state: &mut State) {
    // Une ligne de commentaire si un commentaire est fourni.
    state.comment_lines = match &state.comment {
        Some(c) if !c.is_empty() => 1,
        _ => 0,
    };
    // Une ligne de search bar si la recherche est activee.
    state.search_lines = if state.search { 1 } else { 0 };

    // La liste affiche au minimum 1 ligne.
    let min_visible = state.items_key.len().max(1);
    state.max_visible = state.max_lines.min(min_visible);
    // Total pour remonter le curseur au debut du bloc.
    state.total_lines = state.comment_lines + state.max_visible + state.search_lines;
}
