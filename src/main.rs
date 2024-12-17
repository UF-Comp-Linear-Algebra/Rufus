mod gradescope;

use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about=None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Count {
        #[clap(required = true)]
        filepaths: Vec<Utf8PathBuf>,
    },
}

fn main() {
    let args = Cli::parse();

    match &args.command {
        Commands::Count { filepaths } => {
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
