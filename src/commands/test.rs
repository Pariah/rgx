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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_generated(pattern: &str) -> GenerateResponse {
        GenerateResponse {
            pattern: pattern.to_string(),
            matches: vec![],
            non_matches: vec![],
            explanation: "test pattern".to_string(),
        }
    }

    #[test]
    fn simple_match() {
        let cmd = TestCommand::new("123");
        let gen = make_generated(r"\d+");
        let result = cmd.test_pattern(&gen).unwrap();
        assert!(result.matches);
        let details = result.match_details.unwrap();
        assert_eq!(details.full_match, "123");
    }

    #[test]
    fn no_match() {
        let cmd = TestCommand::new("abc");
        let gen = make_generated(r"\d+");
        let result = cmd.test_pattern(&gen).unwrap();
        assert!(!result.matches);
        assert!(result.match_details.is_none());
    }

    #[test]
    fn partial_match_in_string() {
        let cmd = TestCommand::new("abc123def");
        let gen = make_generated(r"\d+");
        let result = cmd.test_pattern(&gen).unwrap();
        assert!(result.matches);
        let details = result.match_details.unwrap();
        assert_eq!(details.full_match, "123");
        assert_eq!(details.start, 3);
        assert_eq!(details.end, 6);
    }

    #[test]
    fn capture_groups_unnamed() {
        let cmd = TestCommand::new("123-456");
        let gen = make_generated(r"(\d+)-(\d+)");
        let result = cmd.test_pattern(&gen).unwrap();
        assert!(result.matches);
        let details = result.match_details.unwrap();
        assert_eq!(details.groups.len(), 2);
        assert_eq!(details.groups[0].index, 1);
        assert_eq!(details.groups[0].value, "123");
        assert!(details.groups[0].name.is_none());
        assert_eq!(details.groups[1].index, 2);
        assert_eq!(details.groups[1].value, "456");
    }

    #[test]
    fn named_capture_groups() {
        let cmd = TestCommand::new("123");
        let gen = make_generated(r"(?P<digits>\d+)");
        let result = cmd.test_pattern(&gen).unwrap();
        assert!(result.matches);
        let details = result.match_details.unwrap();
        assert_eq!(details.groups.len(), 1);
        assert_eq!(details.groups[0].name, Some("digits".to_string()));
        assert_eq!(details.groups[0].value, "123");
    }

    #[test]
    fn mixed_named_and_unnamed_groups() {
        let cmd = TestCommand::new("abc-123");
        let gen = make_generated(r"([a-z]+)-(?P<num>\d+)");
        let result = cmd.test_pattern(&gen).unwrap();
        assert!(result.matches);
        let details = result.match_details.unwrap();
        assert_eq!(details.groups.len(), 2);
        assert!(details.groups[0].name.is_none());
        assert_eq!(details.groups[0].value, "abc");
        assert_eq!(details.groups[1].name, Some("num".to_string()));
        assert_eq!(details.groups[1].value, "123");
    }

    #[test]
    fn match_position_at_start() {
        let cmd = TestCommand::new("123abc");
        let gen = make_generated(r"\d+");
        let result = cmd.test_pattern(&gen).unwrap();
        let details = result.match_details.unwrap();
        assert_eq!(details.start, 0);
        assert_eq!(details.end, 3);
    }

    #[test]
    fn match_position_at_end() {
        let cmd = TestCommand::new("abc123");
        let gen = make_generated(r"\d+");
        let result = cmd.test_pattern(&gen).unwrap();
        let details = result.match_details.unwrap();
        assert_eq!(details.start, 3);
        assert_eq!(details.end, 6);
    }

    #[test]
    fn full_string_match() {
        let cmd = TestCommand::new("hello");
        let gen = make_generated(r"^hello$");
        let result = cmd.test_pattern(&gen).unwrap();
        assert!(result.matches);
        let details = result.match_details.unwrap();
        assert_eq!(details.full_match, "hello");
        assert_eq!(details.start, 0);
        assert_eq!(details.end, 5);
    }

    #[test]
    fn anchored_no_match() {
        let cmd = TestCommand::new("say hello");
        let gen = make_generated(r"^hello$");
        let result = cmd.test_pattern(&gen).unwrap();
        assert!(!result.matches);
    }

    #[test]
    fn invalid_regex_returns_error() {
        let cmd = TestCommand::new("test");
        let gen = make_generated(r"["); // unclosed bracket
        let result = cmd.test_pattern(&gen);
        assert!(result.is_err());
    }

    #[test]
    fn invalid_regex_unbalanced_parens() {
        let cmd = TestCommand::new("test");
        let gen = make_generated(r"(abc");
        let result = cmd.test_pattern(&gen);
        assert!(result.is_err());
    }

    #[test]
    fn empty_pattern_matches_empty() {
        let cmd = TestCommand::new("");
        let gen = make_generated(r"");
        let result = cmd.test_pattern(&gen).unwrap();
        assert!(result.matches);
        let details = result.match_details.unwrap();
        assert_eq!(details.full_match, "");
    }

    #[test]
    fn empty_pattern_matches_at_start() {
        let cmd = TestCommand::new("abc");
        let gen = make_generated(r"");
        let result = cmd.test_pattern(&gen).unwrap();
        assert!(result.matches);
        let details = result.match_details.unwrap();
        assert_eq!(details.start, 0);
        assert_eq!(details.end, 0);
    }

    #[test]
    fn no_groups_empty_groups_vec() {
        let cmd = TestCommand::new("abc");
        let gen = make_generated(r"abc");
        let result = cmd.test_pattern(&gen).unwrap();
        assert!(result.matches);
        let details = result.match_details.unwrap();
        assert!(details.groups.is_empty());
    }

    #[test]
    fn unicode_in_input() {
        let cmd = TestCommand::new("café");
        let gen = make_generated(r"\w+");
        let result = cmd.test_pattern(&gen).unwrap();
        assert!(result.matches);
    }

    #[test]
    fn test_result_has_pattern() {
        let cmd = TestCommand::new("123");
        let gen = make_generated(r"\d+");
        let result = cmd.test_pattern(&gen).unwrap();
        assert_eq!(result.pattern, r"\d+");
    }

    #[test]
    fn test_result_has_input() {
        let cmd = TestCommand::new("123");
        let gen = make_generated(r"\d+");
        let result = cmd.test_pattern(&gen).unwrap();
        assert_eq!(result.test_input, "123");
    }

    #[test]
    fn test_result_has_generated_response() {
        let cmd = TestCommand::new("123");
        let gen = make_generated(r"\d+");
        let result = cmd.test_pattern(&gen).unwrap();
        assert_eq!(result.generated.explanation, "test pattern");
    }

    #[test]
    fn optional_group_not_matched() {
        let cmd = TestCommand::new("abc");
        let gen = make_generated(r"abc(\d+)?");
        let result = cmd.test_pattern(&gen).unwrap();
        assert!(result.matches);
        let details = result.match_details.unwrap();
        // Optional group didn't match, so groups should be empty
        // (filter_map filters out None)
        assert!(details.groups.is_empty());
    }

    #[test]
    fn alternation_first_branch() {
        let cmd = TestCommand::new("cat");
        let gen = make_generated(r"cat|dog");
        let result = cmd.test_pattern(&gen).unwrap();
        assert!(result.matches);
        assert_eq!(result.match_details.unwrap().full_match, "cat");
    }

    #[test]
    fn alternation_second_branch() {
        let cmd = TestCommand::new("dog");
        let gen = make_generated(r"cat|dog");
        let result = cmd.test_pattern(&gen).unwrap();
        assert!(result.matches);
        assert_eq!(result.match_details.unwrap().full_match, "dog");
    }

    #[test]
    fn email_like_pattern() {
        let cmd = TestCommand::new("test@example.com");
        let gen = make_generated(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}");
        let result = cmd.test_pattern(&gen).unwrap();
        assert!(result.matches);
        assert_eq!(result.match_details.unwrap().full_match, "test@example.com");
    }
}
