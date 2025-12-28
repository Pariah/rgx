mod claude;
mod commands;
mod error;
mod output;

use clap::{CommandFactory, Parser, ValueEnum};
use clap_complete::{generate, Shell};
use commands::explain::ExplainCommand;
use commands::generate::GenerateCommand;
use commands::test::TestCommand;
use commands::Command;
use crossterm::style::Stylize;
use error::{Error, Result};
use std::io;

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum Flavor {
    #[default]
    Rust,
    Js,
    Pcre,
    Posix,
}

impl Flavor {
    fn as_str(&self) -> &'static str {
        match self {
            Flavor::Rust => "rust",
            Flavor::Js => "javascript",
            Flavor::Pcre => "pcre",
            Flavor::Posix => "posix",
        }
    }
}

#[derive(Parser)]
#[command(name = "rgx")]
#[command(about = "Natural language regex patterns powered by Claude")]
#[command(version)]
struct Cli {
    /// Pattern description (generate/test mode) or pattern to explain (explain mode)
    input: Option<String>,

    /// Explain mode: break down an existing regex pattern
    #[arg(short = 'e', long = "explain")]
    explain: bool,

    /// Test mode: generate pattern and test against this input string
    #[arg(short = 't', long = "test", value_name = "INPUT")]
    test: Option<String>,

    /// Output raw JSON
    #[arg(long = "raw")]
    raw: bool,

    /// Regex flavor (affects pattern generation)
    #[arg(long = "flavor", value_enum, default_value_t = Flavor::Rust)]
    flavor: Flavor,

    /// Generate shell completions
    #[arg(long = "completions", value_name = "SHELL")]
    completions: Option<Shell>,
}

fn validate_flags(cli: &Cli) -> Result<()> {
    if cli.explain && cli.test.is_some() {
        return Err(Error::InvalidFlags(
            "Cannot combine -e (explain) and -t (test) flags".to_string(),
        ));
    }
    Ok(())
}

fn run(cli: Cli) -> Result<()> {
    validate_flags(&cli)?;

    let input = cli
        .input
        .ok_or_else(|| Error::InvalidFlags("No input provided".to_string()))?;

    let claude = claude::Claude::default();

    if cli.explain {
        let cmd = ExplainCommand::new();
        let prompt = cmd.build_prompt(&input);
        let response = claude.query(&prompt)?;
        let parsed = cmd.parse_response(&response)?;
        println!("{}", output::format_explain(&parsed, cli.raw));
    } else if let Some(test_input) = &cli.test {
        let gen_cmd = GenerateCommand::new(cli.flavor.as_str());
        let prompt = gen_cmd.build_prompt(&input);
        let response = claude.query(&prompt)?;
        let generated = gen_cmd.parse_response(&response)?;

        let test_cmd = TestCommand::new(test_input);
        let result = test_cmd.test_pattern(&generated)?;
        println!("{}", output::format_test(&result, cli.raw));
    } else {
        let cmd = GenerateCommand::new(cli.flavor.as_str());
        let prompt = cmd.build_prompt(&input);
        let response = claude.query(&prompt)?;
        let parsed = cmd.parse_response(&response)?;
        println!("{}", output::format_generate(&parsed, cli.raw));
    }

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    if let Some(shell) = cli.completions {
        let mut cmd = Cli::command();
        generate(shell, &mut cmd, "rgx", &mut io::stdout());
        return;
    }

    if let Err(e) = run(cli) {
        eprintln!("{}: {}", "error".red(), e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cli(
        input: Option<&str>,
        explain: bool,
        test: Option<&str>,
        raw: bool,
        flavor: Flavor,
    ) -> Cli {
        Cli {
            input: input.map(|s| s.to_string()),
            explain,
            test: test.map(|s| s.to_string()),
            raw,
            flavor,
            completions: None,
        }
    }

    #[test]
    fn validate_flags_generate_mode() {
        let cli = make_cli(Some("email"), false, None, false, Flavor::Rust);
        assert!(validate_flags(&cli).is_ok());
    }

    #[test]
    fn validate_flags_explain_mode() {
        let cli = make_cli(Some(r"\d+"), true, None, false, Flavor::Rust);
        assert!(validate_flags(&cli).is_ok());
    }

    #[test]
    fn validate_flags_test_mode() {
        let cli = make_cli(
            Some("email"),
            false,
            Some("test@example.com"),
            false,
            Flavor::Rust,
        );
        assert!(validate_flags(&cli).is_ok());
    }

    #[test]
    fn validate_flags_explain_and_test_invalid() {
        let cli = make_cli(Some("email"), true, Some("test"), false, Flavor::Rust);
        let result = validate_flags(&cli);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Cannot combine"));
    }

    #[test]
    fn validate_flags_with_raw() {
        let cli = make_cli(Some("email"), false, None, true, Flavor::Rust);
        assert!(validate_flags(&cli).is_ok());
    }

    #[test]
    fn validate_flags_with_js_flavor() {
        let cli = make_cli(Some("email"), false, None, false, Flavor::Js);
        assert!(validate_flags(&cli).is_ok());
    }

    #[test]
    fn validate_flags_with_pcre_flavor() {
        let cli = make_cli(Some("email"), false, None, false, Flavor::Pcre);
        assert!(validate_flags(&cli).is_ok());
    }

    #[test]
    fn validate_flags_with_posix_flavor() {
        let cli = make_cli(Some("email"), false, None, false, Flavor::Posix);
        assert!(validate_flags(&cli).is_ok());
    }

    #[test]
    fn validate_flags_no_input_ok() {
        // validate_flags doesn't check for input - run() does
        let cli = make_cli(None, false, None, false, Flavor::Rust);
        assert!(validate_flags(&cli).is_ok());
    }

    #[test]
    fn flavor_as_str_rust() {
        assert_eq!(Flavor::Rust.as_str(), "rust");
    }

    #[test]
    fn flavor_as_str_js() {
        assert_eq!(Flavor::Js.as_str(), "javascript");
    }

    #[test]
    fn flavor_as_str_pcre() {
        assert_eq!(Flavor::Pcre.as_str(), "pcre");
    }

    #[test]
    fn flavor_as_str_posix() {
        assert_eq!(Flavor::Posix.as_str(), "posix");
    }

    #[test]
    fn flavor_default_is_rust() {
        assert!(matches!(Flavor::default(), Flavor::Rust));
    }
}
