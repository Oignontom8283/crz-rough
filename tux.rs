use crossterm::{
    cursor::{Hide, MoveToColumn, MoveUp, Show},
    event::{self, EnableMouseCapture, DisableMouseCapture, Event, KeyCode, KeyModifiers, MouseEventKind, MouseButton},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use colored::*;
use std::io::{self, Write};
use crate::common::State;

pub fn run_tui(state: &mut State) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let mut stderr = io::stderr();

    let max_visible = 10_usize.min(state.filtered_items.len().max(1));
    let mut scroll_offset: usize = 0;

    let comment_lines: usize = match &state.comment {
        Some(c) if !c.is_empty() => 1,
        _ => 0,
    };
    let search_lines: usize = if state.search { 1 } else { 0 };
    let total_lines = comment_lines + max_visible + search_lines;

    // Position du curseur dans search_query (en indices de char, pas d'octets)
    let mut search_cursor: usize = 0; // 0..=query.chars().count()

    // [1] INIT
    enable_raw_mode()?;
    execute!(stderr, Hide, EnableMouseCapture)?;
    for _ in 0..total_lines {
        writeln!(stderr)?;
    }
    execute!(stderr, MoveUp(total_lines as u16), MoveToColumn(0))?;

    // [2] BOUCLE
    loop {
        render(state, &mut stderr, scroll_offset, max_visible,
               comment_lines, search_lines, search_cursor)?;

        match event::read()? {
        Event::Mouse(mouse) => {
            match mouse.kind {
                MouseEventKind::ScrollUp => {
                    // Même logique que KeyCode::Up
                    if state.cursor_pos > 0 {
                        state.cursor_pos -= 1;
                        if state.cursor_pos < scroll_offset {
                            scroll_offset = state.cursor_pos;
                        } else if scroll_offset > 0 && state.cursor_pos == scroll_offset {
                            scroll_offset = scroll_offset.saturating_sub(1);
                        }
                    }
                }
                MouseEventKind::ScrollDown => {
                    // Même logique que KeyCode::Down
                    let max_idx = state.filtered_items.len().saturating_sub(1);
                    if state.cursor_pos < max_idx {
                        state.cursor_pos += 1;
                        let would_be_last = state.cursor_pos == scroll_offset + max_visible - 1;
                        let more_below = scroll_offset + max_visible < state.filtered_items.len();
                        if state.cursor_pos >= scroll_offset + max_visible {
                            scroll_offset = state.cursor_pos + 1 - max_visible;
                        } else if would_be_last && more_below {
                            scroll_offset += 1;
                        }
                    }
                }
                MouseEventKind::Down(MouseButton::Left) => break,
                _ => {}
            }
        }
        Event::Key(key) => {
            // Ctrl+C
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                cleanup_terminal(total_lines, &mut stderr)?;
                return Ok(None);
            }

            // Ctrl+Backspace : supprimer le mot à gauche du curseur.
            // Crossterm le transmet de plusieurs façons selon le terminal :
            //   - CONTROL + KeyCode::Backspace  (xterm moderne)
            //   - CONTROL + KeyCode::Char('h')  (^H = Backspace ASCII)
            //   - CONTROL + KeyCode::Char('w')  (^W = delete-word readline)
            let is_ctrl_bs = state.search && matches!(
                (key.modifiers.contains(KeyModifiers::CONTROL), &key.code),
                (true, KeyCode::Backspace)
                | (true, KeyCode::Char('h'))
                | (true, KeyCode::Char('w'))
            );
            if is_ctrl_bs {
                search_cursor = delete_word_left(&mut state.search_query, search_cursor);
                update_filter(state);
                clamp_list_cursor(state, &mut scroll_offset, max_visible);
                continue;
            }

            match key.code {
                // Navigation liste
                KeyCode::Up => {
                    if state.cursor_pos > 0 {
                        state.cursor_pos -= 1;
                        // La ligne 0 du viewport est un "..." si scroll_offset > 0 :
                        // le curseur ne doit pas s'y poser → recule le scroll d'1 cran.
                        if state.cursor_pos < scroll_offset {
                            scroll_offset = state.cursor_pos;
                        } else if scroll_offset > 0 && state.cursor_pos == scroll_offset {
                            // curseur sur la ligne qui serait remplacée par "..." en haut
                            scroll_offset = scroll_offset.saturating_sub(1);
                        }
                    }
                }
                KeyCode::Down => {
                    let max_idx = state.filtered_items.len().saturating_sub(1);
                    if state.cursor_pos < max_idx {
                        state.cursor_pos += 1;
                        // La dernière ligne du viewport est un "..." si items dépassent :
                        // le curseur ne doit pas s'y poser → avance le scroll d'1 cran.
                        let would_be_last = state.cursor_pos == scroll_offset + max_visible - 1;
                        let more_below = scroll_offset + max_visible < state.filtered_items.len();
                        if state.cursor_pos >= scroll_offset + max_visible {
                            scroll_offset = state.cursor_pos + 1 - max_visible;
                        } else if would_be_last && more_below {
                            scroll_offset += 1;
                        }
                    }
                }

                // Déplacement curseur search (Left / Right)
                KeyCode::Left if state.search => {
                    if search_cursor > 0 { search_cursor -= 1; }
                }
                KeyCode::Right if state.search => {
                    let len = state.search_query.chars().count();
                    if search_cursor < len { search_cursor += 1; }
                }

                KeyCode::Enter => break,
                KeyCode::Esc => {
                    cleanup_terminal(total_lines, &mut stderr)?;
                    return Ok(None);
                }

                // Backspace : supprime le char à gauche du curseur
                KeyCode::Backspace if state.search => {
                    if search_cursor > 0 {
                        search_cursor = char_delete_left(&mut state.search_query, search_cursor);
                        update_filter(state);
                        clamp_list_cursor(state, &mut scroll_offset, max_visible);
                    }
                }

                // Saisie : insère à la position du curseur
                KeyCode::Char(c) if state.search => {
                    char_insert(&mut state.search_query, search_cursor, c);
                    search_cursor += 1;
                    update_filter(state);
                    state.cursor_pos = 0;
                    scroll_offset = 0;
                }

                _ => {}
            }
        } // Event::Key
        _ => {}
        } // match event
    }

    // [3] NETTOYAGE
    cleanup_terminal(total_lines, &mut stderr)?;

    if let Some(&real_idx) = state.filtered_items.get(state.cursor_pos) {
        Ok(Some(state.item(real_idx).value().to_string()))
    } else {
        Ok(None)
    }
}

// ---------------------------------------------------------------------------
// Helpers édition de texte (opèrent sur indices de char, pas d'octets)
// ---------------------------------------------------------------------------

/// Insère `c` à la position `char_pos` dans `s`.
fn char_insert(s: &mut String, char_pos: usize, c: char) {
    let byte_pos = char_to_byte(s, char_pos);
    s.insert(byte_pos, c);
}

/// Supprime le char à gauche de `char_pos`, retourne le nouveau char_pos.
fn char_delete_left(s: &mut String, char_pos: usize) -> usize {
    if char_pos == 0 { return 0; }
    let byte_end = char_to_byte(s, char_pos);
    let byte_start = char_to_byte(s, char_pos - 1);
    s.drain(byte_start..byte_end);
    char_pos - 1
}

/// Supprime le mot entier à gauche du curseur (Ctrl+Backspace).
/// Retourne le nouveau char_pos.
fn delete_word_left(s: &mut String, char_pos: usize) -> usize {
    if char_pos == 0 { return 0; }
    let chars: Vec<char> = s.chars().collect();

    // Recule en sautant les espaces, puis le mot
    let mut pos = char_pos;
    while pos > 0 && chars[pos - 1] == ' ' { pos -= 1; }
    while pos > 0 && chars[pos - 1] != ' ' { pos -= 1; }

    let byte_start = char_to_byte(s, pos);
    let byte_end   = char_to_byte(s, char_pos);
    s.drain(byte_start..byte_end);
    pos
}

/// Convertit un indice de char en indice d'octet pour `s`.
fn char_to_byte(s: &str, char_pos: usize) -> usize {
    s.char_indices()
        .nth(char_pos)
        .map(|(b, _)| b)
        .unwrap_or(s.len())
}

// ---------------------------------------------------------------------------

fn clamp_list_cursor(state: &mut State, scroll_offset: &mut usize, max_visible: usize) {
    let max_idx = state.filtered_items.len().saturating_sub(1);
    if state.cursor_pos > max_idx { state.cursor_pos = max_idx; }
    if *scroll_offset > state.cursor_pos { *scroll_offset = state.cursor_pos; }
}

fn update_filter(state: &mut State) {
    let query = state.search_query.to_lowercase();
    state.filtered_items = (0..state.items_key.len())
        .filter(|&idx| fuzzy_match(&state.items_key[idx].to_lowercase(), &query))
        .collect();
}

fn fuzzy_match(text: &str, pattern: &str) -> bool {
    if pattern.is_empty() { return true; }
    let mut chars = text.chars();
    for p in pattern.chars() {
        if !chars.any(|c| c == p) { return false; }
    }
    true
}

// ---------------------------------------------------------------------------
// Rendu
// ---------------------------------------------------------------------------

/// Rendu complet du bloc.
///
/// Toutes les lignes terminent par `\n` → le curseur descend de `total_lines`.
/// `MoveUp(total_lines)` ramène exactement à la ligne 0.
///
/// La search bar affiche un curseur bloc (▎ ou fond inversé) à `search_cursor`.
fn render(
    state: &State,
    stderr: &mut io::Stderr,
    scroll_offset: usize,
    max_visible: usize,
    comment_lines: usize,
    search_lines: usize,
    search_cursor: usize,
) -> io::Result<()> {
    let total_lines = comment_lines + max_visible + search_lines;

    // -- Commentaire --
    if comment_lines > 0 {
        if let Some(c) = &state.comment {
            execute!(stderr, MoveToColumn(0), Clear(ClearType::CurrentLine))?;
            writeln!(stderr, "{}", c.bright_black().italic())?;
        }
    }

    // -- Liste --
    let total_items = state.filtered_items.len();
    let has_more_above = scroll_offset > 0;
    let has_more_below = scroll_offset + max_visible < total_items;

    for row in 0..max_visible {
        execute!(stderr, MoveToColumn(0), Clear(ClearType::CurrentLine))?;
        let filtered_idx = scroll_offset + row;

        // Première ligne : "..." si des éléments sont cachés au-dessus
        if row == 0 && has_more_above {
            writeln!(stderr, "{}", "  ...".bright_magenta())?;
        // Dernière ligne : "..." si des éléments sont cachés en-dessous
        } else if row == max_visible - 1 && has_more_below {
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

    // -- Search bar --
    if search_lines > 0 {
        execute!(stderr, MoveToColumn(0), Clear(ClearType::CurrentLine))?;
        let prefix = "~ ".bright_black().to_string();

        if state.search_query.is_empty() {
            // Placeholder : premier char en surbrillance inverse (curseur), reste en gris
            let placeholder = "Fuzzy search here...";
            let cursor_char = &placeholder[..placeholder.char_indices()
                .nth(1).map(|(b,_)| b).unwrap_or(placeholder.len())];
            let rest = &placeholder[cursor_char.len()..];
            writeln!(stderr, "{}{}{}",
                prefix,
                cursor_char.black().on_bright_black(),  // curseur sur placeholder
                rest.bright_black()
            )?;
        } else {
            // Découpe la query en : avant curseur | char sous curseur | après curseur
            let query = &state.search_query;
            let char_count = query.chars().count();
            let cursor_at_end = search_cursor >= char_count;

            let before: String = query.chars().take(search_cursor).collect();
            let (cursor_display, after): (String, String) = if cursor_at_end {
                // Curseur en fin : affiche un espace inversé
                (" ".to_string(), String::new())
            } else {
                let ch: String = query.chars().skip(search_cursor).take(1).collect();
                let rest: String = query.chars().skip(search_cursor + 1).collect();
                (ch, rest)
            };

            writeln!(stderr, "{}{}{}{}",
                prefix,
                before.white(),
                cursor_display.black().on_white(),   // curseur bloc
                after.white()
            )?;
        }
    }

    // Retour à la ligne 0
    execute!(stderr, MoveUp(total_lines as u16), MoveToColumn(0))?;
    stderr.flush()
}

// ---------------------------------------------------------------------------

fn cleanup_terminal(total_lines: usize, stderr: &mut io::Stderr) -> io::Result<()> {
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