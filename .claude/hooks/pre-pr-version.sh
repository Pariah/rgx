#!/bin/bash
# Block PR creation if Cargo.toml version unchanged from base branch

INPUT=$(cat)
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // ""')

# Only check for gh pr create commands
if [[ ! "$COMMAND" =~ "gh pr create" ]]; then
  exit 0
fi

# Determine base branch (main or master)
if git show-ref --verify --quiet refs/heads/main; then
  BASE_BRANCH="main"
elif git show-ref --verify --quiet refs/heads/master; then
  BASE_BRANCH="master"
else
  # No main/master branch, skip check
  exit 0
fi

# Get current version from Cargo.toml
CURRENT=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\([0-9]*\.[0-9]*\.[0-9]*\)".*/\1/')

# Get base branch version
BASE_VERSION=$(git show "$BASE_BRANCH":Cargo.toml 2>/dev/null | grep '^version' | head -1 | sed 's/.*"\([0-9]*\.[0-9]*\.[0-9]*\)".*/\1/')

# If we can't get base version, skip check
if [[ -z "$BASE_VERSION" ]]; then
  exit 0
fi

if [[ "$CURRENT" == "$BASE_VERSION" ]]; then
  cat << EOF
{
  "hookSpecificOutput": {
    "hookEventName": "PreToolUse",
    "permissionDecision": "deny",
    "permissionDecisionReason": "Version unchanged from $BASE_BRANCH ($CURRENT). Bump version in Cargo.toml first, then retry the PR."
  }
}
EOF
  exit 0
fi

exit 0
