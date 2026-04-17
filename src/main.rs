mod cli;
mod extract;
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
        Command::Extract {
            filepaths,
            keys,
            output,
            layout,
            name_by,
            list,
            skip_missing,
            missing_only,
            dry_run,
        } => cli::handlers::handle_extract(
            filepaths,
            keys,
            output,
            layout,
            name_by,
            *list,
            *skip_missing,
            *missing_only,
            *dry_run,
        ),
    }
}
