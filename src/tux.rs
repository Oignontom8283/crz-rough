use crossterm::{
    cursor::{Hide, MoveToColumn, MoveUp, Show},
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use colored::*;
use std::io::{self, Write};

use crate::common::State;

// ! NON FONCTIONNEL, EN COURS DE DÉVELOPPEMENT

pub fn run_tui(state: &mut State) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let mut stderr = io::stderr();
    let mut selected = args.index.min(items.len().saturating_sub(1));
    let height = args.lines.min(items.len());

    // [1] INITIALISATION DU TERMINAL
    enable_raw_mode()?;
    execute!(stderr, Hide)?;
    for _ in 0..height { writeln!(stderr)?; }
    execute!(stderr, MoveUp(height as u16))?;

    // [2] BOUCLE D'ÉVÉNEMENTS (TANT QUE VRAI)
    loop {
        // [2.1] RENDU : POUR CHAQUE LIGNE DE 0 À HEIGHT
        for i in 0..height {
            execute!(stderr, MoveToColumn(0), Clear(ClearType::CurrentLine))?;
            
            // SI index == sélectionné ALORS surbrillance SINON texte normal
            if i == selected {
                let line = format!("> {}", items[i]).cyan().bold();
                write!(stderr, "{}", line)?;
            } else {
                write!(stderr, "  {}", items[i])?;
            }
            
            if i < height - 1 { writeln!(stderr)?; }
        }

        execute!(stderr, MoveUp((height - 1) as u16), MoveToColumn(0))?;
        stderr.flush()?;

        // [2.2] LECTURE ET TRAITEMENT DES ENTRÉES
        if let Event::Key(key) = event::read()? {
            match key.code {
                // SI Touche_Haut ET sélection > 0 ALORS décrémenter sélection
                KeyCode::Up => if selected > 0 { selected -= 1; },

                // SI Touche_Bas ET sélection < max ALORS incrémenter sélection
                KeyCode::Down => if selected < items.len() - 1 && selected < height - 1 { 
                    selected += 1; 
                },

                // SI Touche_Entrée ALORS Sortir de la boucle (Valider)
                KeyCode::Enter => break,

                // SI Touche_Echap OU 'q' ALORS Nettoyer ET retourner Néant (Annuler)
                KeyCode::Esc | KeyCode::Char('q') => {
                    cleanup_terminal(height, &mut stderr)?;
                    return Ok(None);
                }
                _ => {}
            }
        }
    }

    // [3] NETTOYAGE ET RETOUR DU RÉSULTAT
    cleanup_terminal(height, &mut stderr)?;

    if selected < items.len() {
        Ok(Some(items[selected].clone()))
    } else {
        Ok(None)
    }
}

fn cleanup_terminal(height: usize, stderr: &mut io::Stderr) -> io::Result<()> {
    // POUR CHAQUE LIGNE : Effacer et réinitialiser état curseur
    for _ in 0..height {
        execute!(stderr, Clear(ClearType::CurrentLine))?;
        writeln!(stderr)?;
    }
    execute!(stderr, MoveUp(height as u16), MoveToColumn(0), Show)?;
    disable_raw_mode()?;
    Ok(())
}