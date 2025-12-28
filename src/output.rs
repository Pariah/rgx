use crate::commands::explain::ExplainResponse;
use crate::commands::generate::GenerateResponse;
use crate::commands::test::TestResult;
use crossterm::style::Stylize;

/// Colorize a regex pattern for terminal display
pub fn colorize_regex(pattern: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Handle escape sequences
        if chars[i] == '\\' && i + 1 < chars.len() {
            let next = chars[i + 1];
            let escaped = format!("\\{}", next);

            match next {
                // Character classes - cyan
                'd' | 'D' | 'w' | 'W' | 's' | 'S' => {
                    result.push_str(&escaped.cyan().to_string());
                }
                // Anchors - red
                'b' | 'B' | 'A' | 'z' | 'Z' => {
                    result.push_str(&escaped.red().to_string());
                }
                // Other escapes - just show them normally
                _ => {
                    result.push_str(&escaped);
                }
            }
            i += 2;
            continue;
        }

        // Anchors - red
        if chars[i] == '^' || chars[i] == '$' {
            result.push_str(&chars[i].to_string().red().to_string());
            i += 1;
            continue;
        }

        // Quantifiers - yellow
        if chars[i] == '+' || chars[i] == '*' || chars[i] == '?' {
            result.push_str(&chars[i].to_string().yellow().to_string());
            i += 1;
            continue;
        }

        // Alternation - yellow
        if chars[i] == '|' {
            result.push_str(&"|".yellow().to_string());
            i += 1;
            continue;
        }

        // Quantifier braces {n,m} - yellow
        if chars[i] == '{' {
            let mut end = i + 1;
            while end < chars.len() && chars[end] != '}' {
                end += 1;
            }
            if end < chars.len() {
                let quantifier: String = chars[i..=end].iter().collect();
                result.push_str(&quantifier.yellow().to_string());
                i = end + 1;
                continue;
            }
        }

        // Character sets [...] - magenta
        if chars[i] == '[' {
            let mut end = i + 1;
            // Handle negation
            if end < chars.len() && chars[end] == '^' {
                end += 1;
            }
            // Handle ] as first char (literal)
            if end < chars.len() && chars[end] == ']' {
                end += 1;
            }
            // Find closing ]
            while end < chars.len() && chars[end] != ']' {
                if chars[end] == '\\' && end + 1 < chars.len() {
                    end += 2;
                } else {
                    end += 1;
                }
            }
            if end < chars.len() {
                let char_set: String = chars[i..=end].iter().collect();
                result.push_str(&char_set.magenta().to_string());
                i = end + 1;
                continue;
            }
        }

        // Groups - green
        if chars[i] == '(' {
            result.push_str(&"(".green().to_string());
            i += 1;
            continue;
        }
        if chars[i] == ')' {
            result.push_str(&")".green().to_string());
            i += 1;
            continue;
        }

        // Default - no color
        result.push(chars[i]);
        i += 1;
    }

    result
}

pub fn format_generate(resp: &GenerateResponse, raw: bool) -> String {
    if raw {
        return serde_json::to_string_pretty(resp).unwrap_or_default();
    }

    let mut out = String::new();

    out.push_str(&format!("{}\n", "Pattern:".bold()));
    out.push_str(&format!("  {}\n", colorize_regex(&resp.pattern)));

    out.push_str(&format!("\n{}\n", "Explanation:".bold()));
    out.push_str(&format!("  {}\n", resp.explanation));

    out.push_str(&format!("\n{}\n", "Matches:".green().bold()));
    for example in &resp.matches {
        out.push_str(&format!("  {} {}\n", "+".green(), example));
    }

    out.push_str(&format!("\n{}\n", "Non-matches:".red().bold()));
    for example in &resp.non_matches {
        out.push_str(&format!("  {} {}\n", "-".red(), example));
    }

    out
}

pub fn format_explain(resp: &ExplainResponse, raw: bool) -> String {
    if raw {
        return serde_json::to_string_pretty(resp).unwrap_or_default();
    }

    let mut out = String::new();

    out.push_str(&format!("{}\n", "Token Breakdown:".bold()));
    for token in &resp.tokens {
        out.push_str(&format!(
            "  {} {} {}\n",
            colorize_regex(&token.token),
            "→".dark_grey(),
            token.explanation
        ));
    }

    out.push_str(&format!("\n{}\n", "Purpose:".bold()));
    out.push_str(&format!("  {}\n", resp.purpose));

    out
}

pub fn format_test(result: &TestResult, raw: bool) -> String {
    if raw {
        return serde_json::to_string_pretty(result).unwrap_or_default();
    }

    let mut out = String::new();

    out.push_str(&format!("{}\n", "Pattern:".bold()));
    out.push_str(&format!("  {}\n", colorize_regex(&result.pattern)));

    out.push_str(&format!("\n{}\n", "Test Input:".bold()));
    out.push_str(&format!("  \"{}\"\n", result.test_input));

    out.push_str(&format!("\n{} ", "Result:".bold()));
    if result.matches {
        out.push_str(&"MATCH".green().bold().to_string());
        out.push('\n');

        if let Some(details) = &result.match_details {
            out.push_str(&format!(
                "  {} \"{}\" ({}..{})\n",
                "Matched:".dark_grey(),
                details.full_match.clone().green(),
                details.start,
                details.end
            ));
            if !details.groups.is_empty() {
                out.push_str(&format!("  {}\n", "Groups:".dark_grey()));
                for group in &details.groups {
                    let name_str = group
                        .name
                        .as_ref()
                        .map(|n| format!(" ({})", n))
                        .unwrap_or_default();
                    out.push_str(&format!(
                        "    {}{}: \"{}\"\n",
                        group.index,
                        name_str.dark_grey(),
                        group.value.clone().cyan()
                    ));
                }
            }
        }
    } else {
        out.push_str(&"NO MATCH".red().bold().to_string());
        out.push('\n');
    }

    out.push_str(&format!("\n{}\n", "Explanation:".dark_grey()));
    out.push_str(&format!("  {}\n", result.generated.explanation));

    out
}
