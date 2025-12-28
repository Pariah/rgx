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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_flavor_prompt_contains_warning() {
        let cmd = GenerateCommand::new("rust");
        let prompt = cmd.build_prompt("email");
        assert!(prompt.contains("does NOT support lookahead"));
        assert!(prompt.contains("rust"));
    }

    #[test]
    fn js_flavor_prompt() {
        let cmd = GenerateCommand::new("js");
        let prompt = cmd.build_prompt("email");
        assert!(prompt.contains("lookbehind"));
        assert!(prompt.contains("JavaScript"));
    }

    #[test]
    fn pcre_flavor_prompt() {
        let cmd = GenerateCommand::new("pcre");
        let prompt = cmd.build_prompt("email");
        assert!(prompt.contains("full regex features"));
        assert!(prompt.contains("recursion"));
    }

    #[test]
    fn posix_flavor_prompt() {
        let cmd = GenerateCommand::new("posix");
        let prompt = cmd.build_prompt("email");
        assert!(prompt.contains("[0-9]"));
        assert!(prompt.contains("no \\d"));
    }

    #[test]
    fn unknown_flavor_no_note() {
        let cmd = GenerateCommand::new("unknown");
        let prompt = cmd.build_prompt("email");
        // Should still work, just no flavor note
        assert!(prompt.contains("email"));
        assert!(prompt.contains("unknown"));
    }

    #[test]
    fn prompt_includes_description() {
        let cmd = GenerateCommand::new("rust");
        let prompt = cmd.build_prompt("phone number with area code");
        assert!(prompt.contains("phone number with area code"));
    }

    #[test]
    fn prompt_has_json_schema() {
        let cmd = GenerateCommand::new("rust");
        let prompt = cmd.build_prompt("email");
        assert!(prompt.contains("\"pattern\""));
        assert!(prompt.contains("\"matches\""));
        assert!(prompt.contains("\"non_matches\""));
        assert!(prompt.contains("\"explanation\""));
    }

    #[test]
    fn default_flavor_is_rust() {
        let cmd = GenerateCommand::default();
        assert_eq!(cmd.flavor, "rust");
    }

    #[test]
    fn parse_valid_response() {
        let cmd = GenerateCommand::default();
        let json = r#"{"pattern": "\\d+", "matches": ["123", "456"], "non_matches": ["abc"], "explanation": "Matches digits"}"#;
        let resp = cmd.parse_response(json).unwrap();
        assert_eq!(resp.pattern, "\\d+");
        assert_eq!(resp.matches, vec!["123", "456"]);
        assert_eq!(resp.non_matches, vec!["abc"]);
        assert_eq!(resp.explanation, "Matches digits");
    }

    #[test]
    fn parse_malformed_json() {
        let cmd = GenerateCommand::default();
        let result = cmd.parse_response("not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn parse_missing_required_fields() {
        let cmd = GenerateCommand::default();
        let json = r#"{"pattern": "\\d+"}"#;
        // Missing matches, non_matches, explanation - should fail
        let result = cmd.parse_response(json);
        assert!(result.is_err());
    }

    #[test]
    fn parse_empty_arrays() {
        let cmd = GenerateCommand::default();
        let json = r#"{"pattern": ".*", "matches": [], "non_matches": [], "explanation": "Matches anything"}"#;
        let resp = cmd.parse_response(json).unwrap();
        assert!(resp.matches.is_empty());
        assert!(resp.non_matches.is_empty());
    }

    #[test]
    fn generate_response_clone() {
        let resp = GenerateResponse {
            pattern: "\\d+".to_string(),
            matches: vec!["123".to_string()],
            non_matches: vec!["abc".to_string()],
            explanation: "digits".to_string(),
        };
        let cloned = resp.clone();
        assert_eq!(resp.pattern, cloned.pattern);
    }
}
