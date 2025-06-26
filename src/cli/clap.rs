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

        #[arg(long="group-size", short='k', default_value=None, help="Number of emissions that must match to be grouped together.")]
        group_size: Option<usize>,

        #[arg(
            long = "show-emissions",
            short = 'S',
            default_value = "false",
            help = "Show the emissions for each group in the output."
        )]
        show_emissions: bool,

        #[arg(long = "min-size", short = 'm', default_value = "2", value_parser = clap::value_parser!(u64).range(1..), help = "Minimum number of submissions required in a group to be shown.")]
        min_size: u64,

        #[arg(
            long = "exact",
            short = 'E',
            default_value = "false",
            help = "Only show groups that match exactly on k emissions (removes k+1 group submissions from the k groups)."
        )]
        exact: bool,
    },
}
