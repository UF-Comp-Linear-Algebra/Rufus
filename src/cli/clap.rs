use camino::Utf8PathBuf;
use clap::{
    command, crate_authors, crate_description, crate_name, crate_version, Parser, Subcommand,
};

#[derive(Parser)]
#[command(name = crate_name!(), author=crate_authors!())]
#[command(version=crate_version!(), propagate_version=true)]
#[command(about="A tool for detecting plagiarism in Gradescope submissions", long_about=crate_description!())]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
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

        #[arg(long="group-size", short='k', default_value=None)]
        group_size: Option<usize>,

        #[arg(long = "show-emissions", short = 'S', default_value = "false")]
        show_emissions: bool,

        #[arg(long = "min-size", short = 'm', default_value = "2", value_parser = clap::value_parser!(u64).range(1..))]
        min_size: u64,

        #[arg(long = "exact", short = 'E', default_value = "false")]
        exact: bool,
    },

    #[command(about = "Test the emission parsing")]
    Test {
        #[clap(required = true)]
        #[arg(name = "emision")]
        emission: String,
    },
}
