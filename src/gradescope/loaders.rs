use std::{fs, path::Path};

use super::Export;

pub fn load_export<T: AsRef<Path>>(path: T) -> Result<Export, String> {
    fs::read_to_string(path)
        .map_err(|e| e.to_string())
        .and_then(|data| serde_yaml::from_str::<Export>(&data).map_err(|e| e.to_string()))
}
