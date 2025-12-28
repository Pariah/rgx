use crate::commands::generate::GenerateResponse;
use crate::error::Result;
use regex::Regex;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct TestResult {
    pub pattern: String,
    pub test_input: String,
    pub matches: bool,
    pub match_details: Option<MatchDetails>,
    pub generated: GenerateResponse,
}

#[derive(Serialize, Debug)]
pub struct MatchDetails {
    pub full_match: String,
    pub groups: Vec<GroupCapture>,
    pub start: usize,
    pub end: usize,
}

#[derive(Serialize, Debug)]
pub struct GroupCapture {
    pub index: usize,
    pub name: Option<String>,
    pub value: String,
}

pub struct TestCommand {
    pub test_input: String,
}

impl TestCommand {
    pub fn new(test_input: &str) -> Self {
        TestCommand {
            test_input: test_input.to_string(),
        }
    }

    pub fn test_pattern(&self, generated: &GenerateResponse) -> Result<TestResult> {
        let regex = Regex::new(&generated.pattern)?;

        let match_details = regex.captures(&self.test_input).map(|caps| {
            let full = caps.get(0).unwrap();

            let groups: Vec<GroupCapture> = regex
                .capture_names()
                .enumerate()
                .skip(1)
                .filter_map(|(i, name)| {
                    caps.get(i).map(|m| GroupCapture {
                        index: i,
                        name: name.map(|s| s.to_string()),
                        value: m.as_str().to_string(),
                    })
                })
                .collect();

            MatchDetails {
                full_match: full.as_str().to_string(),
                groups,
                start: full.start(),
                end: full.end(),
            }
        });

        Ok(TestResult {
            pattern: generated.pattern.clone(),
            test_input: self.test_input.clone(),
            matches: match_details.is_some(),
            match_details,
            generated: generated.clone(),
        })
    }
}
