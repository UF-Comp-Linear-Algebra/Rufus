use anyhow::Error;
use camino::Utf8PathBuf;
use itertools::Itertools;
use std::{
    collections::BTreeSet,
    fs::File,
    io::{BufRead, BufReader},
};
use walkdir::WalkDir;

use colored::Colorize;

use crate::{
    cli::utils::print_group,
    gradescope::{
        loaders::{load_export, load_exports},
        types::{LatestSubmission, SubmissionTrait, EXPORT_FILENAME},
    },
    rufus::{EmissionsGroup, Grouping},
};

pub fn handle_count(filepaths: &Vec<Utf8PathBuf>) {
    match load_exports(filepaths) {
        Ok(exports) => {
            let count = exports.iter().map(|e| e.len()).sum::<usize>();
            println!("Total submissions: {}", count);
        }
        Err(e) => eprintln!("{}", e),
    }
}

pub fn handle_hunt(
    filepaths: &Vec<Utf8PathBuf>,
    group_size: &Option<usize>,
    show_emissions: &bool,
    min_size: &usize,
    exact: &bool,
) {
    // Grab submissions from loaded exports
    let submissions: Vec<LatestSubmission> = filepaths
        .iter()
        .flat_map(|fp| {
            print!("Parsing file {}... ", fp);
            match load_export(fp) {
                Ok(export) => {
                    println!("{}", "DONE".green());
                    Some(export)
                }
                Err(_) => {
                    println!("{}", "FAILED".red());
                    None
                }
            }
        })
        .flat_map(|e| e.into_values())
        .collect();
    println!();

    // Parse emissions from the submissions
    let emissions = submissions
        .iter()
        .map(|s| s.parse_emissions())
        .collect::<Vec<EmissionsGroup>>();

    let total_emissions = emissions.iter().map(|e| e.len()).sum::<usize>();
    println!(
        "Parsed {} total emissions over {} submissions.\n",
        total_emissions.to_string().bold(),
        submissions.len().to_string().underline()
    );

    // Hunt for groups of submissions with k identical emissions
    let k = group_size.unwrap_or_else(|| emissions.iter().map(|e| e.len()).max().unwrap_or(0));
    print!(
        "Finding groups of emissions (k = {} | exact = {} | min_size = {})... ",
        k.to_string().blue(),
        exact.to_string().blue(),
        min_size.to_string().blue()
    );

    let groups = hunt(&emissions, k, *exact)
        .into_iter()
        .filter(|g| g.len() >= *min_size)
        .sorted_by_key(|g| g.len())
        .rev()
        .collect::<Vec<_>>();
    println!("found {} groups.\n", groups.len().to_string().underline());

    // PRINTING
    for (i, grouping) in groups.iter().enumerate() {
        print_group(i + 1, grouping, *show_emissions);
    }
}

pub fn hunt<'a>(groups: &'a [EmissionsGroup<'a>], k: usize, exact: bool) -> Vec<Grouping<'a>> {
    let all_emission_ids: BTreeSet<&String> =
        groups.iter().flat_map(|g| g.emission_ids()).collect();

    let mut groupings: Vec<Grouping> = vec![];
    for on_ids in all_emission_ids
        .into_iter()
        .combinations(k)
        .map(|ids| ids.into_iter().collect::<BTreeSet<&String>>())
    {
        let mut grouped_for_curr_ids: Vec<Grouping> = vec![];
        for group_a in groups {
            // TODO: don't clone here

            let add_to_grouped = grouped_for_curr_ids
                .iter_mut()
                .find(|grouping| grouping.matches_group_on_ids(group_a, Some(&on_ids), exact));

            match add_to_grouped {
                Some(grouping) => {
                    // If a matching group is found, add to it
                    grouping.add_group(group_a);
                }
                None => {
                    // Otherwise, create a new group with this submission
                    grouped_for_curr_ids.push(Grouping::new(on_ids.clone(), vec![group_a]));
                }
            }
        }

        // Add the groupings for the current set of IDs to the main map
        groupings.extend(grouped_for_curr_ids);
    }

    return groupings;
}

pub fn handle_search(
    submissions_paths: &Vec<Utf8PathBuf>,
    phrase: &String,
    _is_regex: &bool,
) -> () {
    for submissions_path in submissions_paths {
        // Load export file
        let export_path = submissions_path.join(EXPORT_FILENAME);
        print!("Loading export file... ");
        let export = load_export(&export_path);

        match export {
            Ok(e) => {
                println!("DONE");
                println!("Going through {} submissions... ", e.iter().count());

                // Search each directory submission directory recursively
                for submission_dir_name in e.keys() {
                    // Collect the files in the submission directory
                    // TODO: report directories that can't be stat'd
                    let sub_dir = submissions_path.join(submission_dir_name);
                    // println!("Looking in... {}", sub_dir.to_string());
                    let sub_dir_entries: Vec<_> = WalkDir::new(sub_dir)
                        .into_iter()
                        .flatten() // very interesting! we can flatten from Result<T,E>[] to T[]
                        .filter(|f| f.file_type().is_file())
                        .collect();
                    // println!("{} files found!", sub_dir_entries.len());

                    // Search through the files and collect results
                    // TODO: handle that cannot be read
                    let search_results = sub_dir_entries
                        .into_iter()
                        .map(|e| -> Result<(_, Vec<String>), Error> {
                            let f = File::open(e.path())?;
                            let sections = search_file(&f, &phrase, None)?;
                            Ok((e, sections))
                        })
                        .flatten()
                        .filter(|(_, sects)| sects.len() > 0);

                    // Report results
                    for (dir, sects) in search_results {
                        let path = dir.path().to_str();
                        match path {
                            Some(path) => {
                                println!("{}", path.to_string().underline())
                            }
                            None => println!("{}", "???".underline()),
                        }

                        println!("{}\n", sects.join("\n---\n"))
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "Failed to load {} in {} as an export file: {}",
                    EXPORT_FILENAME, submissions_path, e
                );
            }
        }
    }
}

const DEFAULT_DISPLAY_WIDTH: usize = 2;

pub fn search_file(
    file: &File,
    phrase: &String,
    display_width: Option<usize>,
) -> Result<Vec<String>, Error> {
    // TODO: consider better way to deal with default values
    let display_width = display_width.unwrap_or(DEFAULT_DISPLAY_WIDTH);

    // Read in the file as lines
    let lines: Vec<String> = BufReader::new(file).lines().collect::<Result<_, _>>()?;

    // Collect sections where the search phrase is found
    let mut sections: Vec<String> = Vec::new();
    for (line_num, line) in lines.iter().enumerate() {
        // TODO: implement regex
        if line.contains(phrase) {
            let start: usize = line_num.saturating_sub(display_width);
            let end: usize = (line_num + display_width + 1).min(lines.len());

            sections.push(lines[start..end].join("\n"));
        }
    }

    Ok(sections)
}
