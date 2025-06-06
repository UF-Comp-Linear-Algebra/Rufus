mod cli;
mod gradescope;
mod rufus;

use clap::Parser;

use crate::cli::clap::{Cli, Command};

fn main() {
    let args = Cli::parse();

    match &args.command {
        Command::Count { filepaths } => cli::handlers::handle_count(&filepaths),
        Command::Hunt {
            filepaths,
            group_size,
            show_emissions,
            min_size,
            exact,
        } => cli::handlers::handle_hunt(
            &filepaths,
            group_size,
            &show_emissions,
            &(*min_size as usize),
            &exact,
        ),
        Command::Test { emission } => {
            println!("Testing emission parsing for: {}", emission);
            rufus::Emission::parse(&emission)
                .and_then(|e| {
                    println!(
                        "Parsed emission: id = '{}', value = '{}'",
                        e.id(),
                        e.value()
                    );
                    Ok(())
                })
                .unwrap_or_else(|e| {
                    eprintln!("Error parsing emission: {}", e);
                });
        }
    }
}
