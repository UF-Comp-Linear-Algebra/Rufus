use camino::Utf8PathBuf;
use itertools::Itertools;
use std::collections::BTreeSet;

use colored::Colorize;

use crate::{
    cli::utils::print_group,
    extract::{compute_label, decode_value, stringify_yaml_value, Decode, DecodeOutput, KeySpec, Layout, NameBy},
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

pub fn handle_extract(
    filepaths: &Vec<Utf8PathBuf>,
    key_specs: &Vec<KeySpec>,
    output: &Option<Utf8PathBuf>,
    layout: &Layout,
    name_by: &NameBy,
    list: bool,
    skip_missing: bool,
    missing_only: bool,
    dry_run: bool,
) {
    let submissions: Vec<(String, LatestSubmission)> = filepaths
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
        .flat_map(|e| e.into_iter())
        .collect();

    if submissions.is_empty() {
        eprintln!("No submissions loaded.");
        return;
    }
    println!();

    // Collect all extra_data keys present across all submissions
    let all_keys: BTreeSet<String> = submissions
        .iter()
        .flat_map(|(_, s)| s.extra_data_map().into_keys())
        .collect();

    if list {
        println!(
            "Found {} key(s) across {} submission(s):\n",
            all_keys.len(),
            submissions.len()
        );
        for key in &all_keys {
            let count = submissions
                .iter()
                .filter(|(_, s)| s.extra_data_map().contains_key(key))
                .count();
            println!("  {}  ({}/{})", key, count, submissions.len());
        }
        if !all_keys.is_empty() {
            let flags = all_keys
                .iter()
                .map(|k| format!("--key {}", k))
                .collect::<Vec<_>>()
                .join(" ");
            println!("\nSuggested flags:\n  {}", flags);
        }
        return;
    }

    // Resolve which key specs to use: explicit list or all discovered keys with defaults
    let default_specs: Vec<KeySpec>;
    let selected: Vec<&KeySpec> = if key_specs.is_empty() {
        if all_keys.is_empty() {
            println!("No extra_data keys found in any submission.");
            return;
        }
        default_specs = all_keys
            .into_iter()
            .map(|k| KeySpec { key: k, decode: Decode::Raw, ext: None })
            .collect();
        default_specs.iter().collect()
    } else {
        key_specs.iter().collect()
    };

    let mut missing_report: Vec<(String, Vec<String>)> = vec![];
    let mut written: usize = 0;

    for (submission_id, submission) in &submissions {
        let extra_data = submission.extra_data_map();
        let label = compute_label(submission_id, submission.submitters(), name_by);

        let missing: Vec<String> = selected
            .iter()
            .filter(|spec| !extra_data.contains_key(&spec.key))
            .map(|spec| spec.key.clone())
            .collect();

        if !missing.is_empty() {
            missing_report.push((label.clone(), missing));
            if skip_missing {
                continue;
            }
        }

        if missing_only {
            continue;
        }

        if output.is_none() {
            println!("{}:", label.bold());
        }

        for spec in &selected {
            let value = match extra_data.get(&spec.key) {
                Some(v) => stringify_yaml_value(v),
                None => continue,
            };

            let decoded = match decode_value(&value, &spec.decode) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!(
                        "{} decoding '{}' for '{}': {}",
                        "Error".red().bold(),
                        spec.key,
                        label,
                        e
                    );
                    continue;
                }
            };

            match output {
                None => {
                    let display = match &decoded {
                        DecodeOutput::Text(s) => s.as_str().to_string(),
                        DecodeOutput::Binary(b) => format!("[binary, {} bytes]", b.len()),
                    };
                    println!("  {}: {}", spec.key, display);
                }
                Some(out_dir) => {
                    let filename = match &spec.ext {
                        Some(ext) => format!("{}.{}", spec.key, ext),
                        None => spec.key.clone(),
                    };
                    let path = match layout {
                        Layout::Dir => out_dir.join(&label).join(&filename),
                        Layout::Flat => out_dir.join(format!("{}_{}", label, filename)),
                    };
                    if dry_run {
                        println!("[dry-run] {}", path);
                        continue;
                    }
                    if let Some(parent) = path.parent() {
                        if let Err(e) = std::fs::create_dir_all(parent) {
                            eprintln!("{} creating '{}': {}", "Error".red().bold(), parent, e);
                            continue;
                        }
                    }
                    let result = match decoded {
                        DecodeOutput::Text(s) => std::fs::write(&path, s),
                        DecodeOutput::Binary(b) => std::fs::write(&path, b),
                    };
                    match result {
                        Ok(_) => written += 1,
                        Err(e) => eprintln!("{} writing '{}': {}", "Error".red().bold(), path, e),
                    }
                }
            }
        }

        if output.is_none() {
            println!();
        }
    }

    // Report
    if missing_only {
        if missing_report.is_empty() {
            println!(
                "All {} submission(s) have all selected keys.",
                submissions.len()
            );
        } else {
            println!(
                "Missing keys in {} of {} submission(s):\n",
                missing_report.len(),
                submissions.len()
            );
            for (label, keys) in &missing_report {
                println!("  {}: {}", label, keys.join(", "));
            }
        }
    } else {
        if output.is_some() && !dry_run {
            println!("Wrote {} file(s).", written);
        }
        if !missing_report.is_empty() {
            eprintln!(
                "\n{}: {} submission(s) missing key(s):",
                "Warning".yellow().bold(),
                missing_report.len()
            );
            for (label, keys) in &missing_report {
                eprintln!("  {}: {}", label, keys.join(", "));
            }
        }
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
