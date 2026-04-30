use std::io::BufRead;
use clap::Parser;

use crate::{common::State, tux::run_tui};

mod cli;
mod common;
mod input;
mod layout;
mod render;
mod search;
mod scroll;
mod tux;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    // Parse les arguments en ligne de commande (ex: --index, --lines)
    let args = cli::Args::parse();

    // Récupère l'entrée standard (stdin) ligne par ligne (ex: via un pipe `ls | crz-rough`)
    let stdint = std::io::stdin();
    let items = stdint.lock().lines()
        .filter_map(|line| line.ok())
        .collect::<Vec<String>>();

    // Si on n'a rien reçu (pipe vide ou fichier vide), on affiche une erreur et on quitte.
    if items.is_empty() {
        eprintln!("No input provided. Please provide input via stdin.");
        return Ok(());
    }

    // Intialise l'état de l'application
    let mut state = State {
        items_key: items.clone(),
        item_values: None,
        search_query: args.search_default.clone().unwrap_or_default(),
        filtered_items: (0..items.len()).collect(), // All items are displayed initially
        cursor_pos: args.index.min(items.len()).saturating_sub(1), // Index One-based to Zero-Based
        comment: args.comment.clone(),
        search: !args.no_search,
        scroll_offset: 0,
        search_cursor: 0,
        max_lines: args.lines.max(1),
        max_visible: 0,
        comment_lines: 0,
        search_lines: 0,
        total_lines: 0,
    };

    drop(items); // Plus besoin de la list d'origine
    
    // Lance l'interface utilisateur dans le terminal
    if let Some(selected) = run_tui(&mut state)? {

        // Envoyer l'élément de retoure dans le stdout
        println!("{}", selected);
    }
    
    Ok(())
}
