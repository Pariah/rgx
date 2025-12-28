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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colorize_empty_pattern() {
        let result = colorize_regex("");
        assert_eq!(result, "");
    }

    #[test]
    fn colorize_plain_text() {
        let result = colorize_regex("abc");
        assert_eq!(result, "abc");
    }

    #[test]
    fn colorize_character_classes_cyan() {
        // \d, \D, \w, \W, \s, \S should be cyan
        for class in &[r"\d", r"\D", r"\w", r"\W", r"\s", r"\S"] {
            let result = colorize_regex(class);
            // Result should be longer (has ANSI escapes) and contain the class
            assert!(
                result.len() > class.len(),
                "Expected ANSI escapes for {}",
                class
            );
            assert!(result.contains(class), "Expected {} in output", class);
        }
    }

    #[test]
    fn colorize_escaped_anchors_red() {
        // \b, \B, \A, \z, \Z should be red
        for anchor in &[r"\b", r"\B", r"\A", r"\z", r"\Z"] {
            let result = colorize_regex(anchor);
            assert!(
                result.len() > anchor.len(),
                "Expected ANSI escapes for {}",
                anchor
            );
            assert!(result.contains(anchor), "Expected {} in output", anchor);
        }
    }

    #[test]
    fn colorize_line_anchors_red() {
        // ^ and $ should be red
        let caret = colorize_regex("^");
        assert!(caret.len() > 1, "Expected ANSI escapes for ^");

        let dollar = colorize_regex("$");
        assert!(dollar.len() > 1, "Expected ANSI escapes for $");
    }

    #[test]
    fn colorize_quantifiers_yellow() {
        // +, *, ? should be yellow
        for q in &["+", "*", "?"] {
            let result = colorize_regex(q);
            assert!(result.len() > 1, "Expected ANSI escapes for {}", q);
        }
    }

    #[test]
    fn colorize_quantifier_braces_yellow() {
        // {3}, {1,5}, {2,} should be yellow
        for q in &["{3}", "{1,5}", "{2,}"] {
            let result = colorize_regex(q);
            assert!(result.len() > q.len(), "Expected ANSI escapes for {}", q);
            assert!(result.contains(q), "Expected {} in output", q);
        }
    }

    #[test]
    fn colorize_alternation_yellow() {
        let result = colorize_regex("|");
        assert!(result.len() > 1, "Expected ANSI escapes for |");
    }

    #[test]
    fn colorize_character_sets_magenta() {
        // [abc], [^abc], [a-z] should be magenta (entire set)
        for set in &["[abc]", "[^abc]", "[a-z]", "[0-9]"] {
            let result = colorize_regex(set);
            assert!(
                result.len() > set.len(),
                "Expected ANSI escapes for {}",
                set
            );
            assert!(result.contains(set), "Expected {} in output", set);
        }
    }

    #[test]
    fn colorize_character_set_with_escapes() {
        // [\d\w] should be magenta as one unit
        let result = colorize_regex(r"[\d\w]");
        assert!(result.len() > 6, "Expected ANSI escapes for [\\d\\w]");
        assert!(result.contains(r"[\d\w]"), "Set should stay together");
    }

    #[test]
    fn colorize_groups_green() {
        // ( and ) should be green
        let open = colorize_regex("(");
        assert!(open.len() > 1, "Expected ANSI escapes for (");

        let close = colorize_regex(")");
        assert!(close.len() > 1, "Expected ANSI escapes for )");
    }

    #[test]
    fn colorize_non_capturing_group() {
        // (?:...) - parens green, content normal
        let result = colorize_regex("(?:abc)");
        assert!(result.contains("abc"), "Content should be present");
        assert!(result.len() > 7, "Expected ANSI escapes");
    }

    #[test]
    fn colorize_complex_pattern() {
        // ^\d{3}-\d{4}$ - mixed colors
        let result = colorize_regex(r"^\d{3}-\d{4}$");
        // Should be much longer than input due to multiple color escapes
        assert!(result.len() > 15, "Expected many ANSI escapes");
        // Should contain the literal dash
        assert!(result.contains("-"), "Literal dash should be present");
    }

    #[test]
    fn colorize_escaped_brackets_not_treated_as_set() {
        // \[ and \] should NOT trigger character set parsing
        let result = colorize_regex(r"\[abc\]");
        // This should just show the escapes normally, not as a magenta set
        assert!(result.contains(r"\["), "Escaped [ should be present");
        assert!(result.contains(r"\]"), "Escaped ] should be present");
    }

    #[test]
    fn colorize_literal_close_bracket_first_in_set() {
        // []abc] - ] as first char is literal, set is []abc]
        let result = colorize_regex("[]abc]");
        assert!(result.len() > 6, "Expected ANSI escapes");
        assert!(result.contains("[]abc]"), "Set should include literal ]");
    }

    #[test]
    fn colorize_negated_set_with_literal_bracket() {
        // [^]abc] - negated set with ] as first char after ^
        let result = colorize_regex("[^]abc]");
        assert!(result.len() > 7, "Expected ANSI escapes");
        assert!(result.contains("[^]abc]"), "Negated set should be intact");
    }

    #[test]
    fn colorize_unclosed_set_stays_plain() {
        // [abc without closing ] - should not colorize as set
        let result = colorize_regex("[abc");
        // Without closing bracket, it just shows characters
        assert!(result.contains("["), "Opening bracket should be present");
    }

    #[test]
    fn colorize_unclosed_quantifier_brace() {
        // {3 without closing } - should not colorize as quantifier
        let result = colorize_regex("{3");
        assert!(result.contains("{"), "Opening brace should be present");
        assert!(result.contains("3"), "Number should be present");
    }

    #[test]
    fn colorize_other_escapes_not_colored() {
        // \n, \t, \. should pass through without special color
        for esc in &[r"\n", r"\t", r"\."] {
            let result = colorize_regex(esc);
            assert!(result.contains(esc), "Escape {} should pass through", esc);
        }
    }

    #[test]
    fn colorize_email_like_pattern() {
        // Realistic pattern: [a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}
        let pattern = r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}";
        let result = colorize_regex(pattern);
        assert!(result.len() > pattern.len(), "Expected ANSI escapes");
        assert!(result.contains("@"), "Literal @ should be present");
    }
}
