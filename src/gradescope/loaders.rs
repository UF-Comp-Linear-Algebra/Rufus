use std::fs;

use super::Export;

pub fn load_export(path: &str) -> Result<Export, String> {
    fs::read_to_string(path)
        .map_err(|e| e.to_string())
        .and_then(|data| serde_yaml::from_str::<Export>(&data).map_err(|e| e.to_string()))
}
