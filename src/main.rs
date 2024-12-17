use std::time;

use gradescope::LatestSubmission;

mod gradescope;

fn main() {
    let filenames = vec![
        "/Users/robcond/Documents/Code/Rufus/a1_1.yml",
        "/Users/robcond/Documents/Code/Rufus/a1_2.yml",
    ]
    .repeat(5);

    let mut all_submissions: Vec<LatestSubmission> = Vec::new();

    println!("Loading submissions");
    let start = time::Instant::now();
    {
        // load all submissions
        for filename in filenames {
            print!("Loading {}... ", filename);

            let start = time::Instant::now();
            match gradescope::load_export(filename) {
                Ok(export) => {
                    // collect all submissions and push them into a vector (owned)
                    println!(
                        "DONE in {} ms ({} submissions)",
                        start.elapsed().as_millis(),
                        export.len()
                    );
                    all_submissions.extend(export.into_values()); // your submissions are mine now! (i.e. all your base are belong to us)
                }
                Err(e) => eprintln!("ERROR in {} ms\n{}", start.elapsed().as_millis(), e),
            }
        }
    }
    println!(
        "Loaded {} submissions in {} ms",
        all_submissions.len(),
        start.elapsed().as_millis()
    );
}
