use super::Command;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Token {
    pub token: String,
    pub explanation: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ExplainResponse {
    pub tokens: Vec<Token>,
    pub purpose: String,
}

pub struct ExplainCommand;

impl ExplainCommand {
    pub fn new() -> Self {
        ExplainCommand
    }
}

impl Default for ExplainCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for ExplainCommand {
    type Response = ExplainResponse;

    fn build_prompt(&self, pattern: &str) -> String {
        format!(
            r#"Explain this regex pattern token by token: {}

Respond with ONLY valid JSON, no markdown:
{{"tokens": [{{"token": "\\d", "explanation": "Matches any digit 0-9"}}, {{"token": "+", "explanation": "One or more of the preceding"}}], "purpose": "Overall description of what this pattern matches"}}

Requirements:
- Break down EVERY token/component in the pattern
- Group logical units (e.g., keep "[a-z]" together, not "[", "a", "-", "z", "]")
- Keep individual explanations concise (one short sentence each)
- The purpose should be a clear 1-sentence summary of what the entire pattern is for"#,
            pattern
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_contains_pattern() {
        let cmd = ExplainCommand::new();
        let prompt = cmd.build_prompt(r"\d{3}-\d{4}");
        assert!(prompt.contains(r"\d{3}-\d{4}"));
    }

    #[test]
    fn prompt_has_json_schema() {
        let cmd = ExplainCommand::new();
        let prompt = cmd.build_prompt(r"\d+");
        assert!(prompt.contains("\"tokens\""));
        assert!(prompt.contains("\"token\""));
        assert!(prompt.contains("\"explanation\""));
        assert!(prompt.contains("\"purpose\""));
    }

    #[test]
    fn prompt_has_requirements() {
        let cmd = ExplainCommand::new();
        let prompt = cmd.build_prompt(r"\d+");
        assert!(prompt.contains("Break down EVERY token"));
        assert!(prompt.contains("Group logical units"));
    }

    #[test]
    fn default_is_new() {
        let cmd1 = ExplainCommand::new();
        let cmd2 = ExplainCommand::default();
        // Both should produce same prompt for same input
        assert_eq!(cmd1.build_prompt("test"), cmd2.build_prompt("test"));
    }

    #[test]
    fn parse_valid_response() {
        let cmd = ExplainCommand::new();
        let json = r#"{"tokens": [{"token": "\\d", "explanation": "digit"}, {"token": "+", "explanation": "one or more"}], "purpose": "matches digits"}"#;
        let resp = cmd.parse_response(json).unwrap();
        assert_eq!(resp.tokens.len(), 2);
        assert_eq!(resp.tokens[0].token, "\\d");
        assert_eq!(resp.tokens[0].explanation, "digit");
        assert_eq!(resp.tokens[1].token, "+");
        assert_eq!(resp.purpose, "matches digits");
    }

    #[test]
    fn parse_empty_tokens() {
        let cmd = ExplainCommand::new();
        let json = r#"{"tokens": [], "purpose": "empty pattern"}"#;
        let resp = cmd.parse_response(json).unwrap();
        assert!(resp.tokens.is_empty());
        assert_eq!(resp.purpose, "empty pattern");
    }

    #[test]
    fn parse_malformed_json() {
        let cmd = ExplainCommand::new();
        let result = cmd.parse_response("not json");
        assert!(result.is_err());
    }

    #[test]
    fn parse_missing_purpose() {
        let cmd = ExplainCommand::new();
        let json = r#"{"tokens": []}"#;
        let result = cmd.parse_response(json);
        assert!(result.is_err());
    }

    #[test]
    fn parse_missing_tokens() {
        let cmd = ExplainCommand::new();
        let json = r#"{"purpose": "test"}"#;
        let result = cmd.parse_response(json);
        assert!(result.is_err());
    }

    #[test]
    fn token_struct_clone() {
        let token = Token {
            token: "\\d".to_string(),
            explanation: "digit".to_string(),
        };
        let cloned = token.clone();
        assert_eq!(token.token, cloned.token);
        assert_eq!(token.explanation, cloned.explanation);
    }

    #[test]
    fn explain_response_clone() {
        let resp = ExplainResponse {
            tokens: vec![Token {
                token: "a".to_string(),
                explanation: "letter a".to_string(),
            }],
            purpose: "matches a".to_string(),
        };
        let cloned = resp.clone();
        assert_eq!(resp.tokens.len(), cloned.tokens.len());
        assert_eq!(resp.purpose, cloned.purpose);
    }

    #[test]
    fn complex_pattern_in_prompt() {
        let cmd = ExplainCommand::new();
        let pattern = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
        let prompt = cmd.build_prompt(pattern);
        assert!(prompt.contains(pattern));
    }
}
