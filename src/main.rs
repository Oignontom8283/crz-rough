use std::io::BufRead;

use clap::Parser;


mod cli;
mod common;

fn main() {
    
    let args = cli::Args::parse();

    let stdint = std::io::stdin();
    let items = stdint.lock().lines()
        .filter_map(|line| line.ok())
        .collect::<Vec<String>>();

    
}
