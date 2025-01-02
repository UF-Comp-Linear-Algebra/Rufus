use super::gradescope;
use camino::Utf8PathBuf;
use rayon::prelude::*;

fn load_exports(filepaths: &Vec<Utf8PathBuf>) -> Result<Vec<gradescope::Export>, String> {
    // Load the exports in parallel (errors propagate up)
    match filepaths.par_iter().map(gradescope::load_export).collect() {
        Ok(exports) => Ok(exports),
        Err(e) => Err(e.to_string()),
    }
}

pub fn handle_count(filepaths: &Vec<Utf8PathBuf>) {
    match load_exports(filepaths) {
        Ok(exports) => {
            let count = exports.iter().map(|e| e.len()).sum::<usize>();
            println!("Total submissions: {}", count);
        }
        Err(e) => eprintln!("{}", e),
    }
}

pub fn handle_hunt(_filepaths: &Vec<Utf8PathBuf>) {
    eprint!("Not implemented yet");
}
