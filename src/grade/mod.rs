use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

const GRADESCOPE_BASE: &str = "https://www.gradescope.com";

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct GradeState {
    pub course_id: String,
    pub assignment_id: String,
    pub dir: String,
    pub done: HashSet<String>,
}

impl GradeState {
    pub fn new(course_id: String, assignment_id: String, dir: String) -> Self {
        GradeState { course_id, assignment_id, dir, done: HashSet::new() }
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let data = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&data).map_err(|e| e.to_string())
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let data = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, data).map_err(|e| e.to_string())
    }

    pub fn mark_done(&mut self, key: String) {
        self.done.insert(key);
    }

    pub fn unmark_done(&mut self, key: &str) {
        self.done.remove(key);
    }

    pub fn is_done(&self, key: &str) -> bool {
        self.done.contains(key)
    }

    pub fn is_consistent(&self, course_id: &str, assignment_id: &str, dir: &str) -> bool {
        self.course_id == course_id && self.assignment_id == assignment_id && self.dir == dir
    }
}

pub fn done_key(submission_id: &str) -> String {
    submission_id.to_string()
}

/// Parses the numeric Gradescope submission ID from a directory name.
/// Expects the `submission_{digits}` format produced by `extract --name-by submission_id`.
pub fn parse_submission_id(dir_name: &str) -> Option<String> {
    dir_name
        .strip_prefix("submission_")
        .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))
        .map(str::to_string)
}

pub fn build_url(course_id: &str, assignment_id: &str, submission_id: &str) -> String {
    format!(
        "{}/courses/{}/assignments/{}/submissions/{}",
        GRADESCOPE_BASE, course_id, assignment_id, submission_id
    )
}

/// Substitutes `{dir}` and `{file:name}` placeholders in a command template.
pub fn substitute_cmd(template: &str, dir: &str) -> String {
    let s = template.replace("{dir}", dir);
    let re = regex::Regex::new(r"\{file:([^}]+)\}").unwrap();
    re.replace_all(&s, |caps: &regex::Captures| format!("{}/{}", dir, &caps[1]))
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse_submission_id ---

    #[test]
    fn parse_id_valid() {
        assert_eq!(
            parse_submission_id("submission_403094485"),
            Some("403094485".to_string())
        );
    }

    #[test]
    fn parse_id_missing_prefix() {
        assert_eq!(parse_submission_id("403094485"), None);
    }

    #[test]
    fn parse_id_non_numeric_suffix() {
        assert_eq!(parse_submission_id("submission_abc"), None);
    }

    #[test]
    fn parse_id_empty_suffix() {
        assert_eq!(parse_submission_id("submission_"), None);
    }

    #[test]
    fn parse_id_mixed_suffix() {
        assert_eq!(parse_submission_id("submission_123abc"), None);
    }

    // --- build_url ---

    #[test]
    fn url_builds_correctly() {
        assert_eq!(
            build_url("1217863", "7420607", "403094485"),
            "https://www.gradescope.com/courses/1217863/assignments/7420607/submissions/403094485"
        );
    }

    // --- substitute_cmd ---

    #[test]
    fn cmd_substitutes_dir() {
        assert_eq!(
            substitute_cmd("echo {dir}", "/tmp/foo"),
            "echo /tmp/foo"
        );
    }

    #[test]
    fn cmd_substitutes_file() {
        assert_eq!(
            substitute_cmd("picotool load {file:level_uf2.uf2}", "/tmp/foo"),
            "picotool load /tmp/foo/level_uf2.uf2"
        );
    }

    #[test]
    fn cmd_substitutes_both() {
        assert_eq!(
            substitute_cmd("cp {file:out.bin} {dir}/backup.bin", "/tmp/foo"),
            "cp /tmp/foo/out.bin /tmp/foo/backup.bin"
        );
    }

    #[test]
    fn cmd_no_placeholders_unchanged() {
        assert_eq!(substitute_cmd("ls -la", "/tmp/foo"), "ls -la");
    }

    // --- done_key ---

    #[test]
    fn done_key_is_submission_id() {
        assert_eq!(done_key("403094485"), "403094485");
    }

    // --- GradeState ---

    #[test]
    fn state_mark_and_check_done() {
        let mut s = GradeState::new("c".into(), "a".into(), "/tmp".into());
        assert!(!s.is_done("403094485"));
        s.mark_done("403094485".to_string());
        assert!(s.is_done("403094485"));
    }

    #[test]
    fn state_unmark_done() {
        let mut s = GradeState::new("c".into(), "a".into(), "/tmp".into());
        s.mark_done("403094485".to_string());
        s.unmark_done("403094485");
        assert!(!s.is_done("403094485"));
    }

    #[test]
    fn state_is_consistent_matching() {
        let s = GradeState::new("1217863".into(), "7420607".into(), "/tmp/uf2s".into());
        assert!(s.is_consistent("1217863", "7420607", "/tmp/uf2s"));
    }

    #[test]
    fn state_is_consistent_different_course() {
        let s = GradeState::new("1217863".into(), "7420607".into(), "/tmp/uf2s".into());
        assert!(!s.is_consistent("9999999", "7420607", "/tmp/uf2s"));
    }

    #[test]
    fn state_is_consistent_different_assignment() {
        let s = GradeState::new("1217863".into(), "7420607".into(), "/tmp/uf2s".into());
        assert!(!s.is_consistent("1217863", "9999999", "/tmp/uf2s"));
    }

    #[test]
    fn state_is_consistent_different_dir() {
        let s = GradeState::new("1217863".into(), "7420607".into(), "/tmp/uf2s".into());
        assert!(!s.is_consistent("1217863", "7420607", "/tmp/other"));
    }

    #[test]
    fn state_roundtrips_json() {
        let mut s = GradeState::new("1217863".into(), "7420607".into(), "/tmp/uf2s".into());
        s.mark_done("403094485".to_string());

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("state.json");
        s.save(&path).unwrap();
        let loaded = GradeState::load(&path).unwrap();
        assert_eq!(s, loaded);
    }
}
