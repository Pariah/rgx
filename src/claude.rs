use crate::error::{Error, Result};
use serde::Deserialize;
use std::process::Command;

#[derive(Deserialize, Debug)]
pub struct ClaudeResponse {
    pub result: String,
    #[allow(dead_code)]
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub is_error: bool,
}

pub struct Claude {
    model: String,
}

impl Claude {
    pub fn new(model: &str) -> Self {
        Claude {
            model: model.to_string(),
        }
    }

    pub fn query(&self, prompt: &str) -> Result<String> {
        let output = Command::new("claude")
            .args([
                "-p",
                prompt,
                "--model",
                &self.model,
                "--output-format",
                "json",
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Claude(stderr.to_string()));
        }

        let response: ClaudeResponse = serde_json::from_slice(&output.stdout)?;

        if response.is_error {
            return Err(Error::Claude(response.result));
        }

        Ok(strip_markdown_code_block(&response.result))
    }
}

fn strip_markdown_code_block(s: &str) -> String {
    let trimmed = s.trim();

    if trimmed.starts_with("```") {
        let without_start = if let Some(rest) = trimmed.strip_prefix("```json") {
            rest
        } else if let Some(rest) = trimmed.strip_prefix("```") {
            rest
        } else {
            return s.to_string();
        };

        if let Some(content) = without_start.strip_suffix("```") {
            return content.trim().to_string();
        }
    }

    s.to_string()
}

impl Default for Claude {
    fn default() -> Self {
        Claude::new("haiku")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_json_code_block() {
        let input = "```json\n{\"key\": \"value\"}\n```";
        assert_eq!(strip_markdown_code_block(input), "{\"key\": \"value\"}");
    }

    #[test]
    fn strip_plain_code_block() {
        let input = "```\n{\"key\": \"value\"}\n```";
        assert_eq!(strip_markdown_code_block(input), "{\"key\": \"value\"}");
    }

    #[test]
    fn no_stripping_for_clean_json() {
        let input = "{\"key\": \"value\"}";
        assert_eq!(strip_markdown_code_block(input), input);
    }

    #[test]
    fn handles_leading_trailing_whitespace() {
        let input = "  ```json\n{}\n```  ";
        assert_eq!(strip_markdown_code_block(input), "{}");
    }

    #[test]
    fn incomplete_code_block_missing_close() {
        // Missing closing ``` - returns original
        let input = "```json\n{}";
        assert_eq!(strip_markdown_code_block(input), input);
    }

    #[test]
    fn multiline_json_content() {
        let input = "```json\n{\n  \"a\": 1,\n  \"b\": 2\n}\n```";
        let expected = "{\n  \"a\": 1,\n  \"b\": 2\n}";
        assert_eq!(strip_markdown_code_block(input), expected);
    }

    #[test]
    fn empty_code_block() {
        let input = "```json\n```";
        assert_eq!(strip_markdown_code_block(input), "");
    }

    #[test]
    fn just_backticks_no_content() {
        let input = "``````";
        assert_eq!(strip_markdown_code_block(input), "");
    }

    #[test]
    fn plain_text_not_code_block() {
        let input = "just some text";
        assert_eq!(strip_markdown_code_block(input), input);
    }

    #[test]
    fn claude_new_sets_model() {
        let claude = Claude::new("sonnet");
        assert_eq!(claude.model, "sonnet");
    }

    #[test]
    fn claude_default_uses_haiku() {
        let claude = Claude::default();
        assert_eq!(claude.model, "haiku");
    }

    #[test]
    fn parse_claude_response_valid() {
        let json = r#"{"result": "hello", "session_id": "abc", "is_error": false}"#;
        let resp: ClaudeResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.result, "hello");
        assert!(!resp.is_error);
    }

    #[test]
    fn parse_claude_response_missing_optional_fields() {
        // session_id and is_error have #[serde(default)]
        let json = r#"{"result": "hello"}"#;
        let resp: ClaudeResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.result, "hello");
        assert_eq!(resp.session_id, "");
        assert!(!resp.is_error);
    }

    #[test]
    fn parse_claude_response_error_flag() {
        let json = r#"{"result": "error message", "is_error": true}"#;
        let resp: ClaudeResponse = serde_json::from_str(json).unwrap();
        assert!(resp.is_error);
    }
}
