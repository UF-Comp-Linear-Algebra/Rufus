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
        Command::Search {
            submissions_paths,
            phrase,
            is_regex,
        } => cli::handlers::handle_search(submissions_paths, phrase, is_regex),
    }
}
