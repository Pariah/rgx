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
