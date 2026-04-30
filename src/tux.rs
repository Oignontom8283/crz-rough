use crossterm::{
    cursor::{Hide, MoveToColumn, MoveUp},
    event::{self, EnableMouseCapture},
    execute,
    terminal::enable_raw_mode,
};
use std::io::{self, Write};

use crate::{
    common::State,
    input::{handle_event, InputAction},
    layout, render,
};

pub fn run_tui(state: &mut State) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let mut stderr = io::stderr();

    // Calcule le layout (lignes visibles et total).
    layout::init_layout(state);

    // [1] INIT
    enable_raw_mode()?;
    execute!(stderr, Hide, EnableMouseCapture)?;
    for _ in 0..state.total_lines {
        writeln!(stderr)?;
    }
    execute!(stderr, MoveUp(state.total_lines as u16), MoveToColumn(0))?;

    // [2] BOUCLE
    loop {
        render::render(state, &mut stderr)?;

        // Lit un event et applique l'action correspondante.
        match handle_event(state, event::read()?) {
            InputAction::Quit => {
                render::cleanup_terminal(state.total_lines, &mut stderr)?;
                return Ok(None);
            }
            InputAction::Select => break,
            InputAction::None => {}
        }
    }

    // [3] NETTOYAGE
    render::cleanup_terminal(state.total_lines, &mut stderr)?;

    if let Some(&real_idx) = state.filtered_items.get(state.cursor_pos) {
        Ok(Some(state.item(real_idx).value().to_string()))
    } else {
        Ok(None)
    }
}