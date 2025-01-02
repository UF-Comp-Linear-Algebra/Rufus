mod cli;
mod gradescope;

use camino::Utf8PathBuf;
use clap::{crate_authors, crate_description, crate_name, crate_version, Parser, Subcommand};

#[derive(Parser)]
#[command(name = crate_name!(), author=crate_authors!())]
#[command(version=crate_version!(), propagate_version=true)]
#[command(about="A tool for detecting plagiarism in Gradescope submissions", long_about=crate_description!())]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(about = "Count the number of submissions in the given export files")]
    Count {
        #[clap(required = true)]
        #[arg(name = "export files")]
        filepaths: Vec<Utf8PathBuf>,
    },

    #[command(about = "Detect plagiarism in the given export files")]
    Hunt {
        #[clap(required = true)]
        #[arg(name = "export files")]
        filepaths: Vec<Utf8PathBuf>,
    },
}

fn main() {
    let args = Cli::parse();

    match &args.command {
        Command::Count { filepaths } => cli::handle_count(filepaths),
        Command::Hunt { filepaths } => cli::handle_hunt(filepaths),
    }
}
