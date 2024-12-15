use serde::Deserialize;
use std::{collections::HashMap, fs};

pub type Export = HashMap<String, Submission>;

pub fn load_export(path: &String) -> Result<Export, String> {
    fs::read_to_string(path)
        .map_err(|e| e.to_string())
        .and_then(|data| serde_yaml::from_str::<Export>(&data).map_err(|e| e.to_string()))
}

#[derive(Deserialize, Debug)]
pub struct Submission {
    #[serde(rename = ":submitters")]
    submitters: Vec<Submitter>,

    #[serde(rename = ":created_at")]
    created_at: String,

    #[serde(rename = ":score")]
    score: Score,

    #[serde(rename = ":status")]
    status: String,

    #[serde(rename = ":results")]
    results: Results,

    #[serde(rename = ":history", skip_deserializing)] // todo: implement this
    history: Vec<Submission>, // note: historical submissions have a different format
}

#[derive(Deserialize, Debug)]

pub struct Submitter {
    #[serde(rename = ":name")]
    name: String,

    #[serde(rename = ":sid")]
    sid: Option<String>,

    #[serde(rename = ":email")]
    email: String,
}

pub type Score = f32;

#[derive(Deserialize, Debug)]
pub struct Results {
    score: Score,
    tests: Vec<Test>,

    output: Option<String>,
    extra_data: Option<()>,
    visibility: String,
    leaderboard: Vec<()>,
    output_format: Option<String>,
    execution_time: f32,
    test_name_format: Option<String>,
    test_output_format: Option<String>,
}

#[derive(Deserialize, Debug)]

pub struct Test {
    name: String,
    tags: Option<Vec<String>>,
    score: Option<Score>,
    number: String,
    output: String,
    status: String,
    max_score: Option<Score>,
    visibility: Option<Visibility>,
    name_format: Option<OutputFormat>,
    output_format: Option<OutputFormat>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Visibility {
    Hidden,
    AfterDueDate,
    AfterPublished,
    Visible,
}

#[derive(Deserialize, Debug)]
pub enum OutputFormat {
    #[serde(rename = "text")]
    Text,

    #[serde(rename = "html")]
    HTML,

    #[serde(rename = "simple_format")]
    SimpleFormat,

    #[serde(rename = "md")]
    Markdown,

    #[serde(rename = "ansi")]
    ANSI,
}

#[derive(Deserialize, Debug)]
pub struct Leaderboard {
    name: String,
    value: LeaderboardValue,
    order: Option<SortOrder>,
}

#[derive(Deserialize, Debug)]
pub enum LeaderboardValue {
    Float(f32),
    String(String),
}

#[derive(Deserialize, Debug)]
pub enum SortOrder {
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    Descending,
}
