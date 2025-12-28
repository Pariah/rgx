use super::Command;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GenerateResponse {
    pub pattern: String,
    pub matches: Vec<String>,
    pub non_matches: Vec<String>,
    pub explanation: String,
}

#[derive(Clone)]
pub struct GenerateCommand {
    pub flavor: String,
}

impl GenerateCommand {
    pub fn new(flavor: &str) -> Self {
        GenerateCommand {
            flavor: flavor.to_string(),
        }
    }
}

impl Default for GenerateCommand {
    fn default() -> Self {
        Self::new("rust")
    }
}

impl Command for GenerateCommand {
    type Response = GenerateResponse;

    fn build_prompt(&self, description: &str) -> String {
        let flavor_note = match self.flavor.as_str() {
            "rust" => "Note: Rust regex does NOT support lookahead/lookbehind. Use only supported features.",
            "js" => "JavaScript regex supports lookahead but not lookbehind in all environments.",
            "pcre" => "PCRE supports full regex features including lookahead, lookbehind, and recursion.",
            "posix" => "POSIX regex is limited - no \\d, \\w shortcuts. Use character classes like [0-9], [a-zA-Z_].",
            _ => "",
        };

        format!(
            r#"Generate a regex pattern that matches: "{}"

Target regex flavor: {}
{}

Respond with ONLY valid JSON, no markdown:
{{"pattern": "the regex pattern", "matches": ["example1", "example2", "example3"], "non_matches": ["non-match1", "non-match2"], "explanation": "Brief explanation of how the pattern works"}}

Requirements:
- The pattern should be valid for the {} regex flavor
- Provide 2-3 realistic example strings that WILL match
- Provide 1-2 realistic example strings that will NOT match
- Keep the explanation concise (1-2 sentences)"#,
            description, self.flavor, flavor_note, self.flavor
        )
    }
}
