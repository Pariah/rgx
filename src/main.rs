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
