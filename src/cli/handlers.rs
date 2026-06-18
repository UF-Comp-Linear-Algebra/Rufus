use anyhow::Error;
use camino::Utf8PathBuf;
use itertools::Itertools;
use std::{
    collections::BTreeSet,
    fs::File,
    io::{BufRead, BufReader},
};
use walkdir::WalkDir;
use std::io::{self, Write};

use colored::Colorize;

use crate::{
    cli::utils::print_group,
    extract::{compute_label, decode_value, stringify_yaml_value, Decode, DecodeOutput, KeySpec, Layout, NameBy},
    grade::{done_key, parse_submission_id, substitute_cmd, GradeState},
    gradescope::{
        loaders::{load_export, load_exports},
        types::{LatestSubmission, Submitter, SubmissionTrait, EXPORT_FILENAME},
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
    alongside: bool,
) {
    // Build batches: (effective_output_dir, submissions).
    // alongside=true → one batch per source file, output dir = file's parent.
    // alongside=false → one batch of all submissions, output dir = --output or None (display).
    let batches: Vec<(Option<Utf8PathBuf>, Vec<(String, LatestSubmission)>)> = if alongside {
        filepaths
            .iter()
            .filter_map(|fp| {
                print!("Parsing file {}... ", fp);
                match load_export(fp) {
                    Ok(export) => {
                        println!("{}", "DONE".green());
                        let out_dir = fp
                            .parent()
                            .map(|p| Utf8PathBuf::from(p))
                            .unwrap_or_else(|| Utf8PathBuf::from("."));
                        Some((Some(out_dir), export.into_iter().collect()))
                    }
                    Err(_) => {
                        println!("{}", "FAILED".red());
                        None
                    }
                }
            })
            .collect()
    } else {
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
        vec![(output.clone(), submissions)]
    };

    let total_submissions: usize = batches.iter().map(|(_, s)| s.len()).sum();
    if total_submissions == 0 {
        eprintln!("No submissions loaded.");
        return;
    }
    println!();

    // Collect all extra_data keys across all batches
    let all_keys: BTreeSet<String> = batches
        .iter()
        .flat_map(|(_, subs)| subs.iter().flat_map(|(_, s)| s.extra_data_map().into_keys()))
        .collect();

    if list {
        let all_subs: Vec<&LatestSubmission> =
            batches.iter().flat_map(|(_, subs)| subs.iter().map(|(_, s)| s)).collect();
        println!(
            "Found {} key(s) across {} submission(s):\n",
            all_keys.len(),
            all_subs.len()
        );
        for key in &all_keys {
            let count = all_subs.iter().filter(|s| s.extra_data_map().contains_key(key)).count();
            println!("  {}  ({}/{})", key, count, all_subs.len());
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

    // Resolve key specs once: explicit list or all discovered keys
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
    let display_mode = !alongside && output.is_none();

    for (out_dir, submissions) in &batches {
        for (submission_id, submission) in submissions {
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

            if display_mode {
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

                if display_mode {
                    let display = match &decoded {
                        DecodeOutput::Text(s) => s.as_str().to_string(),
                        DecodeOutput::Binary(b) => format!("[binary, {} bytes]", b.len()),
                    };
                    println!("  {}: {}", spec.key, display);
                } else {
                    let out_dir = out_dir.as_ref().unwrap();
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

            if display_mode {
                println!();
            }
        }
    }

    // Report
    if missing_only {
        if missing_report.is_empty() {
            println!("All {} submission(s) have all selected keys.", total_submissions);
        } else {
            println!(
                "Missing keys in {} of {} submission(s):\n",
                missing_report.len(),
                total_submissions
            );
            for (label, keys) in &missing_report {
                println!("  {}: {}", label, keys.join(", "));
            }
        }
    } else {
        if !display_mode && !dry_run {
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

// ── grade ────────────────────────────────────────────────────────────────────

struct Step {
    submission_id: String,
    dir: Utf8PathBuf,
    sub_idx: usize,
}

fn run_cmd(template: &str, dir: &str) {
    let cmd = substitute_cmd(template, dir);
    println!("Running: {}", cmd.italic());

    #[cfg(unix)]
    let result = std::process::Command::new("sh").args(["-c", &cmd]).status();
    #[cfg(windows)]
    let result = std::process::Command::new("cmd").args(["/C", &cmd]).status();

    match result {
        Ok(s) if !s.success() => eprintln!(
            "{}: command exited with {}",
            "Warning".yellow().bold(),
            s
        ),
        Err(e) => eprintln!("{} running command: {}", "Error".red().bold(), e),
        _ => {}
    }
}

pub fn handle_grade(
    dir_arg: &Option<Utf8PathBuf>,
    export: &Option<Utf8PathBuf>,
    cmd: &Option<String>,
    state_path: &Option<Utf8PathBuf>,
    reset: bool,
) {
    // Resolve state file path: explicit > <dir>/.rufus-grade
    let state_file = state_path.clone().or_else(|| {
        dir_arg.as_ref().map(|d| d.join(".rufus-grade"))
    });

    // Try loading existing state to fill in missing params
    let existing_state: Option<GradeState> = state_file.as_ref().and_then(|p| {
        if !reset && p.exists() {
            match GradeState::load(p.as_std_path()) {
                Ok(s) => Some(s),
                Err(e) => {
                    eprintln!("{} loading state: {}", "Error".red().bold(), e);
                    None
                }
            }
        } else {
            None
        }
    });

    // Resolve dir: CLI arg takes priority, state fills gap
    let dir: Utf8PathBuf = match dir_arg.clone()
        .or_else(|| existing_state.as_ref().map(|s| Utf8PathBuf::from(&s.dir)))
    {
        Some(d) => d,
        None => {
            eprintln!("{}: <dir> is required on first run (no state file found).", "Error".red().bold());
            return;
        }
    };

    // Now that we have dir, resolve state file if it wasn't explicit
    let state_file = state_file.unwrap_or_else(|| dir.join(".rufus-grade"));

    // Load submitter info from export if provided
    let submitter_map: std::collections::HashMap<String, Vec<Submitter>> =
        if let Some(export_path) = export {
            match load_export(export_path) {
                Ok(exp) => exp
                    .into_iter()
                    .filter_map(|(k, v)| {
                        parse_submission_id(&k).map(|id| (id, v.submitters().clone()))
                    })
                    .collect(),
                Err(e) => {
                    eprintln!("{} loading export: {}", "Error".red().bold(), e);
                    std::collections::HashMap::new()
                }
            }
        } else {
            std::collections::HashMap::new()
        };

    // Walk and sort submission directories
    let mut submissions: Vec<(String, Utf8PathBuf)> = match std::fs::read_dir(&dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                let path = Utf8PathBuf::from(e.path().to_string_lossy().as_ref());
                parse_submission_id(&name).map(|id| (id, path))
            })
            .collect(),
        Err(e) => {
            eprintln!("{} reading directory '{}': {}", "Error".red().bold(), dir, e);
            return;
        }
    };
    submissions.sort_by(|a, b| a.0.cmp(&b.0));

    if submissions.is_empty() {
        eprintln!("No submission directories found in '{}'.", dir);
        return;
    }

    // Validate consistency if resuming, then use or create state
    let mut state = match existing_state {
        Some(s) => {
            if !s.is_consistent(dir.as_str()) {
                eprintln!(
                    "{}: state file is from a different session.",
                    "Error".red().bold()
                );
                eprintln!("  Saved dir:   {}", s.dir);
                eprintln!("  Current dir: {}", dir);
                eprintln!("Use --reset to start over.");
                return;
            }
            println!("Resuming — {} submission(s) already done.\n", s.done.len());
            s
        }
        None => GradeState::new(dir.to_string()),
    };

    let steps: Vec<Step> = submissions
        .iter()
        .enumerate()
        .map(|(si, (sub_id, sub_dir))| Step {
            submission_id: sub_id.clone(),
            dir: sub_dir.clone(),
            sub_idx: si,
        })
        .collect();

    let n_subs = submissions.len();
    let mut cursor: usize = 0;
    let mut last_cmd_sub: Option<String> = None;

    while cursor < steps.len() {
        let step = &steps[cursor];
        let key = done_key(&step.submission_id);

        if state.is_done(&key) {
            cursor += 1;
            continue;
        }

        // Header
        println!(
            "\n{}",
            format!(
                "=== {}  ({} / {}) ===",
                step.submission_id,
                step.sub_idx + 1,
                n_subs
            )
            .bold()
        );
        if let Some(subs) = submitter_map.get(&step.submission_id) {
            for s in subs {
                let sid = s.sid.as_deref().unwrap_or("no SID");
                println!("    {}  ·  {}  ·  SID {}", s.name, s.email, sid);
            }
        }
        println!();

        // Run command once per submission
        if let Some(template) = cmd {
            if last_cmd_sub.as_deref() != Some(&step.submission_id) {
                run_cmd(template, step.dir.as_str());
                last_cmd_sub = Some(step.submission_id.clone());
                println!();
            }
        }

        // Prompt loop
        let show_rerun = cmd.is_some();
        loop {
            println!();
            if show_rerun {
                print!("[Enter] done  [s] skip  [r] re-run  [b] back  [q] quit\n> ");
            } else {
                print!("[Enter] done  [s] skip  [b] back  [q] quit\n> ");
            }
            io::stdout().flush().ok();

            let mut input = String::new();
            io::stdin().lock().read_line(&mut input).ok();

            match input.trim() {
                "" => {
                    state.mark_done(key.clone());
                    if let Err(e) = state.save(state_file.as_std_path()) {
                        eprintln!("{} saving state: {}", "Warning".yellow().bold(), e);
                    }
                    cursor += 1;
                    break;
                }
                "s" => {
                    cursor += 1;
                    break;
                }
                "r" => {
                    if let Some(template) = cmd {
                        run_cmd(template, step.dir.as_str());
                    } else {
                        println!("No --cmd specified.");
                    }
                }
                "b" => {
                    if cursor == 0 {
                        println!("Already at the first submission.");
                    } else {
                        cursor -= 1;
                        let prev_key = done_key(&steps[cursor].submission_id);
                        state.unmark_done(&prev_key);
                        // Also reset last_cmd_sub so the command re-runs
                        last_cmd_sub = None;
                        if let Err(e) = state.save(state_file.as_std_path()) {
                            eprintln!("{} saving state: {}", "Warning".yellow().bold(), e);
                        }
                    }
                    break;
                }
                "q" => {
                    println!("\nProgress saved to '{}'.", state_file);
                    return;
                }
                _ => println!("Unknown input. Try Enter, s, r, b, or q."),
            }
        }
    }

    println!(
        "\n{} All {} submission(s) graded.",
        "Done!".green().bold(),
        n_subs
    );
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
                println!("Going through {} submissions... ", e.len());

                // Search each directory submission directory recursively
                for submission_dir_name in e.keys() {
                    // Collect the files in the submission directory
                    // TODO: report directories that can't be stat'd
                    let sub_dir = submissions_path.join(submission_dir_name);
                    // println!("Looking in... {}", sub_dir.to_string());
                    let sub_dir_entries: Vec<_> = WalkDir::new(&sub_dir)
                        .into_iter()
                        .filter_map(|entry| match entry {
                            Ok(e) => Some(e),
                            Err(err) => {
                                eprintln!("Warning: failed to read entry under {}: {}", sub_dir, err);
                                None
                            }
                        })
                        .filter(|f| f.file_type().is_file())
                        .collect();
                    // println!("{} files found!", sub_dir_entries.len());

                    // Search through the files and collect results
                    // TODO: handle that cannot be read
                    let search_results = sub_dir_entries.into_iter().filter_map(|entry| {
                        match File::open(entry.path()) {
                            Ok(f) => match search_file(&f, phrase, None) {
                                Ok(sections) if !sections.is_empty() => Some((entry, sections)),
                                Ok(_) => None,
                                Err(err) => {
                                    eprintln!(
                                        "Warning: failed to search {}: {}",
                                        entry.path().display(),
                                        err
                                    );
                                    None
                                }
                            },
                            Err(err) => {
                                eprintln!(
                                    "Warning: failed to open {}: {}",
                                    entry.path().display(),
                                    err
                                );
                                None
                            }
                        }
                    });

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
