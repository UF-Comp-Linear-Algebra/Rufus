use crate::rufus::Grouping;
use colored::Colorize;
use itertools::Itertools;

pub fn print_group(group_num: usize, grouping: &Grouping, show_emissions: bool) {
    let submitters: Vec<_> = grouping
        .groups()
        .iter()
        .flat_map(|g| g.submission().submitters())
        .sorted_by(|a, b| a.name.cmp(&b.name))
        .collect();

    println!("{}", format!("Group {}:", group_num).bold());
    for (i, submitter) in submitters.iter().enumerate() {
        // Print the submitter's name and ID
        println!(
            "\t{} {} (SID: {})",
            format!("({})", i + 1).bold(),
            submitter.name,
            submitter.sid.as_ref().unwrap_or(&"Unknown SID".to_string())
        );
    }

    // Print emissions that matched the grouping criteria
    if show_emissions {
        print!("\n\t{}", format!("Emissions:").underline());
        if let Some(first) = grouping.groups().first() {
            for id in grouping.on_ids() {
                if let Some(some_emission) = first.emissions_map().get(*id) {
                    println!();
                    println!("\t\"{}\"", some_emission.id().italic());
                    println!("\t{}", some_emission.value().replace('\n', "\n\t").blue());
                } else {
                    println!("\tEmission not found for ID: {}", id);
                }
            }
        } else {
            println!("\n\tNo groups available to display emissions.");
        }
    }
    println!();
}
