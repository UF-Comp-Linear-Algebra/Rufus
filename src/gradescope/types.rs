use serde::Deserialize;
use std::collections::HashMap;

use crate::rufus::{Emission, EmissionParseError, EmissionsGroup};

pub type Export = HashMap<String, LatestSubmission>;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Submission {
    Latest(LatestSubmission),
    Historical(HistoricalSubmission),
}

static EMISSION_NUMBER_PREFIX: &str = "99.";

pub trait SubmissionTrait {
    fn submitters(&self) -> &Vec<Submitter>;
    fn created_at(&self) -> &String;
    fn score(&self) -> &Score;
    fn status(&self) -> &String;
    fn results(&self) -> &Option<Results>;

    fn parse_emissions<'a>(&'a self) -> EmissionsGroup<'a>
    where
        Self: Sized,
    {
        // pub fn parse_emissions<'a>(&'a self) -> EmissionsGroup<'a> {
        match self.results().as_ref() {
            Some(Results::Processed(processed_results)) => {
                let emissions: Vec<Result<Emission, EmissionParseError>> = processed_results
                    .tests
                    .iter()
                    .filter(|t| t.number.starts_with(EMISSION_NUMBER_PREFIX) && t.output.is_some())
                    .flat_map(|t| {
                        t.output
                            .as_ref()
                            .unwrap() // unwrap because we just checked it's Some
                            .lines()
                            .map(|line| Emission::parse(line))
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>();

                // Filter out any errors in parsing emissions
                let emissions: Vec<Emission> = emissions
                    .into_iter()
                    .filter_map(|e| match e {
                        Ok(emission) => Some(emission),
                        Err(e) => {
                            eprintln!("Error parsing emission: {}", e);
                            None
                        }
                    })
                    .collect();

                EmissionsGroup::new(self, emissions)
            }
            _ => EmissionsGroup::new(self, vec![]),
        }
        // }
    }
}

#[derive(Deserialize, Debug)]
pub struct LatestSubmission {
    #[serde(rename = ":submitters")]
    pub submitters: Vec<Submitter>,

    #[serde(rename = ":created_at")]
    pub created_at: String,

    #[serde(rename = ":score")]
    pub score: Score,

    #[serde(rename = ":status")]
    pub status: String, // should be an ENUM

    #[serde(rename = ":results")]
    pub results: Option<Results>, // this should only be in the case where the status is "unprocessed" (or "processing" or "failed", i guess)

    #[serde(rename = ":history")]
    pub history: Vec<HistoricalSubmission>,
}

#[derive(Deserialize, Debug)]
pub struct HistoricalSubmission {
    #[serde(rename = ":submitters")]
    pub submitters: Vec<Submitter>,

    #[serde(rename = ":created_at")]
    pub created_at: String,

    #[serde(rename = ":score")]
    pub score: Score,

    #[serde(rename = ":status")]
    pub status: String,

    #[serde(rename = ":results")]
    pub results: Option<Results>,

    #[serde(rename = ":id")]
    pub id: u32,
}

impl SubmissionTrait for HistoricalSubmission {
    fn submitters(&self) -> &Vec<Submitter> {
        &self.submitters
    }

    fn created_at(&self) -> &String {
        &self.created_at
    }

    fn score(&self) -> &Score {
        &self.score
    }

    fn status(&self) -> &String {
        &self.status
    }

    fn results(&self) -> &Option<Results> {
        &self.results
    }
}

impl SubmissionTrait for LatestSubmission {
    fn submitters(&self) -> &Vec<Submitter> {
        &self.submitters
    }

    fn created_at(&self) -> &String {
        &self.created_at
    }

    fn score(&self) -> &Score {
        &self.score
    }

    fn status(&self) -> &String {
        &self.status
    }

    fn results(&self) -> &Option<Results> {
        &self.results
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Submitter {
    #[serde(rename = ":name")]
    pub name: String,

    #[serde(rename = ":sid")]
    pub sid: Option<String>,

    #[serde(rename = ":email")]
    pub email: String,
}

pub type Score = f32;
#[derive(Deserialize, Debug)]
pub struct ProcessedResults {
    pub score: Score,
    pub tests: Vec<Test>,

    pub output: Option<String>,
    pub extra_data: Option<serde_yaml::Value>,
    pub visibility: String,
    pub leaderboard: Vec<LeaderboardItem>,
    pub output_format: Option<String>,
    pub execution_time: f32,
    pub test_name_format: Option<String>,
    pub test_output_format: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct FailedResults {
    pub output: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Results {
    Processed(ProcessedResults),
    Failed(FailedResults),
}

#[derive(Deserialize, Debug)]
pub struct Test {
    pub name: String,
    pub tags: Option<Vec<String>>,
    pub score: Option<Score>,
    pub number: String,
    pub output: Option<String>,
    pub status: String,
    pub max_score: Option<Score>,
    pub extra_data: Option<serde_yaml::Value>,
    pub visibility: Option<Visibility>,
    pub name_format: Option<OutputFormat>,
    pub output_format: Option<OutputFormat>,
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
pub struct LeaderboardItem {
    pub name: String,
    pub value: LeaderboardValue,
    pub order: Option<SortOrder>,
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
