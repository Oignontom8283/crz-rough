use colored::*;
use crossterm::{
    cursor::{MoveToColumn, MoveUp, Show},
    event::DisableMouseCapture,
    execute,
    terminal::{disable_raw_mode, Clear, ClearType},
};
use std::io::{self, Write};

use crate::common::State;

pub fn render(state: &State, stderr: &mut io::Stderr) -> io::Result<()> {
    let total_lines = state.comment_lines + state.max_visible + state.search_lines;

    // -- Commentaire --
    if state.comment_lines > 0 {
        if let Some(c) = &state.comment {
            execute!(stderr, MoveToColumn(0), Clear(ClearType::CurrentLine))?;
            writeln!(stderr, "{}", c.bright_black().italic())?;
        }
    }

    // -- Liste --
    let total_items = state.filtered_items.len();
    let has_more_above = state.scroll_offset > 0;
    let has_more_below = state.scroll_offset + state.max_visible < total_items;

    for row in 0..state.max_visible {
        execute!(stderr, MoveToColumn(0), Clear(ClearType::CurrentLine))?;
        let filtered_idx = state.scroll_offset + row;

        // Premiere ligne: "..." si des elements sont caches au-dessus.
        if row == 0 && has_more_above {
            writeln!(stderr, "{}", "  ...".bright_magenta())?;
        // Derniere ligne: "..." si des elements sont caches en-dessous.
        } else if row == state.max_visible - 1 && has_more_below {
            writeln!(stderr, "{}", "  ...".bright_magenta())?;
        } else if filtered_idx < total_items {
            let real_idx = state.filtered_items[filtered_idx];
            let item = state.item(real_idx);
            let label = item.label();
            if filtered_idx == state.cursor_pos {
                writeln!(stderr, "{}", format!("> {}", label).cyan().bold())?;
            } else {
                writeln!(stderr, "  {}", label)?;
            }
        } else {
            writeln!(stderr)?;
        }
    }

    if state.search_lines > 0 {
        execute!(stderr, MoveToColumn(0), Clear(ClearType::CurrentLine))?;
        let prefix = "~ ".bright_black().to_string();

        if state.search_query.is_empty() {
            // Placeholder: premier char en surbrillance inverse (curseur).
            let placeholder = "Fuzzy search here...";
            let cursor_char = &placeholder[..placeholder
                .char_indices()
                .nth(1)
                .map(|(b, _)| b)
                .unwrap_or(placeholder.len())];
            let rest = &placeholder[cursor_char.len()..];
            writeln!(stderr, "{}{}{}", prefix, cursor_char.black().on_bright_black(), rest.bright_black())?;
        } else {
            let query = &state.search_query;
            let char_count = query.chars().count();
            let cursor_at_end = state.search_cursor >= char_count;

            // Decoupe la query en: avant curseur | char sous curseur | apres curseur.
            let before: String = query.chars().take(state.search_cursor).collect();
            let (cursor_display, after): (String, String) = if cursor_at_end {
                (" ".to_string(), String::new())
            } else {
                let ch: String = query.chars().skip(state.search_cursor).take(1).collect();
                let rest: String = query.chars().skip(state.search_cursor + 1).collect();
                (ch, rest)
            };

            writeln!(stderr, "{}{}{}{}", prefix, before.white(), cursor_display.black().on_white(), after.white())?;
        }
    }

    // Retour a la ligne 0 du bloc.
    execute!(stderr, MoveUp(total_lines as u16), MoveToColumn(0))?;
    stderr.flush()
}

pub fn cleanup_terminal(total_lines: usize, stderr: &mut io::Stderr) -> io::Result<()> {
    // Nettoie le bloc et remet le terminal en mode normal.
    for i in 0..total_lines {
        execute!(stderr, MoveToColumn(0), Clear(ClearType::CurrentLine))?;
        if i < total_lines - 1 {
            writeln!(stderr)?;
        }
    }
    if total_lines > 1 {
        execute!(stderr, MoveUp((total_lines - 1) as u16))?;
    }
    execute!(stderr, DisableMouseCapture, MoveToColumn(0), Show)?;
    disable_raw_mode()?;
    Ok(())
}
