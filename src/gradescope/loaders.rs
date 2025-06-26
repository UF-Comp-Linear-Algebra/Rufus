use std::{fs, path::Path};

use camino::Utf8PathBuf;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::gradescope::types::Export;

pub fn load_export<T: AsRef<Path>>(path: T) -> Result<Export, String> {
    fs::read_to_string(path)
        .map_err(|e| e.to_string())
        .and_then(|data| serde_yaml::from_str::<Export>(&data).map_err(|e| e.to_string()))
}

pub fn load_exports(filepaths: &Vec<Utf8PathBuf>) -> Result<Vec<Export>, String> {
    // Load the exports in parallel and aggregate errors
    let results: Vec<Result<Export, String>> = filepaths.par_iter().map(load_export).collect();
    
    let mut exports = Vec::new();
    let mut errors = Vec::new();
    
    for result in results {
        match result {
            Ok(export) => exports.push(export),
            Err(err) => errors.push(err),
        }
    }
    
    if !errors.is_empty() {
        Err(errors.join("; "))
    } else {
        Ok(exports)
    }
}
