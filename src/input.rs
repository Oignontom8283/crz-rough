use crossterm::event::{Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind};

use crate::{common::State, scroll, search};

pub enum InputAction {
	None,
	Quit,
	Select,
}

pub fn handle_event(state: &mut State, event: Event) -> InputAction {
	// Traduit un event clavier/souris en action logique.
	match event {
		Event::Mouse(mouse) => match mouse.kind {
			MouseEventKind::ScrollUp => {
				// Meme logique que la touche Up.
				scroll::move_up(state);
				InputAction::None
			}
			MouseEventKind::ScrollDown => {
				// Meme logique que la touche Down.
				scroll::move_down(state);
				InputAction::None
			}
			MouseEventKind::Down(MouseButton::Left) => InputAction::Select,
			_ => InputAction::None,
		},
		Event::Key(key) => {
			// Ctrl+C => annule
			if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
				return InputAction::Quit;
			}

			// Ctrl+Backspace: supprime le mot a gauche du curseur.
			let is_ctrl_bs = state.search
				&& matches!(
					(key.modifiers.contains(KeyModifiers::CONTROL), &key.code),
					(true, KeyCode::Backspace)
						| (true, KeyCode::Char('h'))
						| (true, KeyCode::Char('w'))
				);
			if is_ctrl_bs {
				state.search_cursor =
					search::delete_word_left(&mut state.search_query, state.search_cursor);
				search::update_filter(state);
				scroll::clamp_list_cursor(state);
				return InputAction::None;
			}

			match key.code {
				// Navigation liste
				KeyCode::Up => {
					scroll::move_up(state);
					InputAction::None
				}
				KeyCode::Down => {
					scroll::move_down(state);
					InputAction::None
				}

				// Deplacement curseur dans la search bar
				KeyCode::Left if state.search => {
					if state.search_cursor > 0 {
						state.search_cursor -= 1;
					}
					InputAction::None
				}
				KeyCode::Right if state.search => {
					let len = state.search_query.chars().count();
					if state.search_cursor < len {
						state.search_cursor += 1;
					}
					InputAction::None
				}
				KeyCode::Enter => InputAction::Select,
				KeyCode::Esc => InputAction::Quit,
				KeyCode::Backspace if state.search => {
					// Supprime le char a gauche du curseur
					if state.search_cursor > 0 {
						state.search_cursor = search::char_delete_left(
							&mut state.search_query,
							state.search_cursor,
						);
						search::update_filter(state);
						scroll::clamp_list_cursor(state);
					}
					InputAction::None
				}
				KeyCode::Char(c) if state.search => {
					// Saisie: insere a la position du curseur
					search::char_insert(&mut state.search_query, state.search_cursor, c);
					state.search_cursor += 1;
					search::update_filter(state);
					state.cursor_pos = 0;
					state.scroll_offset = 0;
					InputAction::None
				}
				_ => InputAction::None,
			}
		}
		_ => InputAction::None,
	}
}
