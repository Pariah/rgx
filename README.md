# RGX(1)

## NAME

**rgx** - natural language regex patterns powered by Claude

## SYNOPSIS

```
rgx [-e | -t INPUT] [--flavor FLAVOR] [--raw] PATTERN
```

## DESCRIPTION

Generate regex patterns from natural language descriptions, explain existing patterns token-by-token, or test patterns against input strings.

Requires `claude` CLI in PATH.

## OPTIONS

| Flag | Description |
|------|-------------|
| `-e, --explain` | Explain mode: break down existing pattern |
| `-t, --test INPUT` | Test mode: generate pattern and test against INPUT |
| `--flavor FLAVOR` | Regex flavor: rust, js, pcre, posix (default: rust) |
| `--raw` | Output JSON |
| `--completions SHELL` | Generate shell completions |

Flags `-e` and `-t` are mutually exclusive.

## MODES

**Generate** (default): Describe what to match in plain English. Returns pattern with examples of matches and non-matches.

**Explain**: Provide an existing regex pattern. Returns token-by-token breakdown with overall purpose.

**Test**: Describe what to match, provide test string. Pattern is generated then tested locally using Rust regex crate. Shows match result and captured groups.

## EXAMPLES

```
rgx "email address"
rgx "US phone number" --flavor js
rgx -e '\d{3}-\d{4}'
rgx -e '^[a-f0-9]{8}-[a-f0-9]{4}'
rgx -t "foo@bar.com" "email address"
rgx -t "2024-01-15" "ISO date"
rgx --raw "uuid"
```

## INSTALLATION

```
cargo install --path .
```

## SEE ALSO

claude(1), regex(7)
