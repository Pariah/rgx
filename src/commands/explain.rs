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
