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
}

fn main() {
    let args = Cli::parse();

    match &args.command {
        Command::Count { filepaths } => {
            let mut total_count: usize = 0;

            for path in filepaths {
                let fname = path.file_name().unwrap_or("");

                match gradescope::load_export(&path) {
                    Ok(export) => {
                        println!("{}: {}", fname, export.len());
                        total_count += export.len();
                    }
                    Err(e) => {
                        eprintln!("{}: {}", fname, e);
                    }
                }
            }

            println!("=> Total: {} submissions", total_count);
        }
    }
}
