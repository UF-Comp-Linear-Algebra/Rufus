use camino::Utf8PathBuf;
use itertools::Itertools;
use std::collections::BTreeSet;

use colored::Colorize;

use crate::{
    cli::utils::print_group,
    gradescope::{
        loaders::{load_export, load_exports},
        types::{LatestSubmission, SubmissionTrait},
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

    /* TODO: Implement --exact logic */
    if *exact {
        panic!("Exact mode is not implemented yet!");
    }

    let groups = hunt(&emissions, k, false)
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

pub fn hunt<'a>(groups: &'a [EmissionsGroup<'a>], k: usize, _exact: bool) -> Vec<Grouping<'a>> {
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
                .find(|grouping| grouping.matches_group_on_ids(group_a, Some(&on_ids)));

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
