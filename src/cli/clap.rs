use camino::Utf8PathBuf;
use clap::{
    crate_authors, crate_description, crate_name, crate_version, Parser, Subcommand,
};

use crate::extract::{parse_key_spec, KeySpec, Layout, NameBy};


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

    #[command(about = "Extract extra_data fields from Gradescope export files")]
    Extract {
        #[clap(required = true)]
        #[arg(name = "export files")]
        filepaths: Vec<Utf8PathBuf>,

        #[arg(
            long = "key",
            value_name = "NAME[:DECODE[:EXT]]",
            value_parser = parse_key_spec,
            help = "Key to extract. Format: name[:decode[:ext]] (e.g. level_uf2:base64:uf2, checksum::sha256). Repeatable. Default: all keys."
        )]
        keys: Vec<KeySpec>,

        #[arg(
            long,
            short = 'o',
            help = "Output directory. If omitted, values are printed to stdout instead of written to files."
        )]
        output: Option<Utf8PathBuf>,

        #[arg(
            long,
            default_value = "dir",
            help = "Output layout: 'dir' creates a subdirectory per submission; 'flat' uses prefixed filenames"
        )]
        layout: Layout,

        #[arg(
            long,
            default_value = "submission_id",
            value_name = "FIELD",
            help = "How to name submissions: submission_id, name, email, or sid"
        )]
        name_by: NameBy,

        #[arg(long, default_value = "false", help = "List all available keys across submissions and exit")]
        list: bool,

        #[arg(
            long,
            default_value = "false",
            conflicts_with = "missing_only",
            help = "Skip submissions that are missing any selected key"
        )]
        skip_missing: bool,

        #[arg(
            long,
            default_value = "false",
            conflicts_with = "skip_missing",
            help = "Only report which submissions are missing selected keys; do not write files"
        )]
        missing_only: bool,

        #[arg(long, default_value = "false", help = "Show what would be written without writing anything")]
        dry_run: bool,

        #[arg(
            long,
            short = 'A',
            default_value = "false",
            conflicts_with = "output",
            help = "Write files alongside each source YAML (implies file output)"
        )]
        alongside: bool,
    },

    #[command(about = "Walk extracted submissions and run grading commands")]
    Grade {
        #[arg(help = "Directory of extracted submissions. Required on first run; loaded from state file on resume.")]
        dir: Option<Utf8PathBuf>,

        #[arg(long, short = 'e', help = "Gradescope export YAML to show submitter names")]
        export: Option<Utf8PathBuf>,

        #[arg(
            long,
            short = 'x',
            value_name = "TEMPLATE",
            help = "Command to run once per submission. Supports {dir} and {file:name} placeholders."
        )]
        cmd: Option<String>,

        #[arg(long, short = 's', help = "Resume state file (default: <dir>/.rufus-grade)")]
        state: Option<Utf8PathBuf>,

        #[arg(long, help = "Ignore existing state and start over")]
        reset: bool,
    },

    // TODO: provide help info regarding how metadata file is the source-of-truth
    // TODO: case-insensitive
    // TODO: multiple phrases
    // TODO: regex
    // TODO: non-typable unicode
    #[command(about = "Search for a phrase in student submissions files (as plain-text)")]
    Search {
        #[clap(required = true)]
        #[arg(name = "submissions directories")]
        submissions_paths: Vec<Utf8PathBuf>,

        #[clap(required = true)]
        #[arg(name = "phrase", help = "Phrase to search for in student files")]
        // TODO: implement multiple phrases
        phrase: String,

        #[arg(
            long = "pattern",
            short = 'P',
            default_value = "false",
            help = "Interpret phrase as a regex pattern"
        )]
        is_regex: bool,
    },
}
